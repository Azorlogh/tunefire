mod probe;

use std::{
	collections::{HashMap, VecDeque},
	pin::Pin,
	sync::{Arc, RwLock},
	time::{Duration, Instant},
};

use anyhow::anyhow;
use hubdj_core::{UserId, UserToken};
use tf_plugin::{Plugin, SourcePlugin};
use tokio::{sync::mpsc, task, time};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{codegen::futures_core::Stream, transport::Server, Request, Response, Status};

pub mod pb {
	tonic::include_proto!("hubdj");
}

use pb::{hubdj_server::HubdjServer, AuthRequest, AuthResponse, SubmitPlaylistRequest};
use url::Url;

type HResult<T> = Result<Response<T>, Status>;

pub struct User {
	id: UserId,
	token: UserToken,
	name: String,
	queue: Option<VecDeque<pb::Track>>,
}

pub struct Booth {
	dj: UserId,
	track: pb::Track,
	started_at: Instant,
	// queue: Vec<pb::Track>,
	user_queue: VecDeque<UserId>,
	duration: Duration,
}

#[derive(Clone)]
pub struct MyServer {
	users: Arc<RwLock<HashMap<UserId, User>>>,
	booth: Arc<RwLock<Option<Booth>>>,
	booth_listeners: Arc<RwLock<Vec<tokio::sync::mpsc::Sender<Result<pb::Booth, Status>>>>>,
}

impl Default for MyServer {
	fn default() -> Self {
		Self {
			users: Default::default(),
			booth: Default::default(),
			booth_listeners: Default::default(),
		}
	}
}

fn get_booth_state(users: &HashMap<UserId, User>, booth: &Booth) -> pb::booth::Playing {
	pb::booth::Playing {
		dj: booth.dj.0,
		track: Some(booth.track.clone()),
		queue: booth
			.user_queue
			.iter()
			.map(|uid| {
				users
					.get(uid)
					.unwrap()
					.queue
					.as_ref()
					.unwrap()
					.front()
					.unwrap()
					.clone()
			})
			.collect(),
	}
}

impl MyServer {
	pub async fn send_booth_state(&self) -> Result<(), Box<dyn std::error::Error>> {
		let booth = pb::Booth {
			playing: self
				.booth
				.read()
				.unwrap()
				.as_ref()
				.map(|booth| get_booth_state(&self.users.read().unwrap(), booth)),
		};
		for sender in self.booth_listeners.read().unwrap().iter() {
			sender.send(Result::Ok(booth.clone())).await?;
		}
		Ok(())
	}
}

#[tonic::async_trait]
impl pb::hubdj_server::Hubdj for MyServer {
	async fn auth(&self, request: Request<AuthRequest>) -> HResult<AuthResponse> {
		let (id, token) = (UserId(rand::random()), UserToken(rand::random()));

		let mut users = self.users.write().unwrap();

		users.insert(
			id,
			User {
				id,
				token,
				name: request.into_inner().name,
				queue: None,
			},
		);

		let users = users.keys().map(|cid| cid.0).collect();
		let res = AuthResponse {
			token: token.0,
			id: id.0,
			users,
		};
		Ok(Response::new(res))
	}

	async fn get_user(&self, request: Request<pb::UserId>) -> HResult<pb::User> {
		let id = UserId(request.into_inner().id);
		let users = self.users.read().unwrap();
		let user = users
			.get(&id)
			.ok_or(Status::not_found("user does not exist"))?;
		Ok(Response::new(pb::User {
			id: id.0,
			name: user.name.clone(),
			queue: user.queue.clone().map(|q| pb::user::Queue {
				tracks: Vec::from(q),
			}),
		}))
	}

	type StreamBoothStateStream = Pin<Box<dyn Stream<Item = Result<pb::Booth, Status>> + Send>>;
	async fn stream_booth_state(
		&self,
		_request: Request<()>,
	) -> HResult<Self::StreamBoothStateStream> {
		let (tx, rx) = mpsc::channel(128);
		let booth = self.booth.clone();
		let users = self.users.clone();
		self.booth_listeners.write().unwrap().push(tx.clone());
		tokio::spawn(async move {
			let booth = {
				let users = users.read().unwrap();
				pb::Booth {
					playing: booth
						.read()
						.unwrap()
						.as_ref()
						.map(|booth| get_booth_state(&users, booth)),
				}
			};
			if let Err(_) = tx.send(Result::Ok(booth)).await {
				println!("oops");
			}
		});
		let output_stream = ReceiverStream::new(rx);
		Ok(Response::new(
			Box::pin(output_stream) as Self::StreamBoothStateStream
		))
	}

	async fn submit_playlist(
		&self,
		request: Request<SubmitPlaylistRequest>,
	) -> HResult<pb::Status> {
		let args = request.into_inner();
		let mut users = self.users.write().unwrap();
		let user = users.get_mut(&UserId(args.token));
		if let Some(user) = user {
			user.queue = Some(VecDeque::from_iter(
				args.playlist.unwrap().tracks.into_iter(),
			));
		}
		Ok(Response::new(pb::Status { ok: true }))
	}
}

pub struct Runner {
	plugins: Arc<RwLock<Vec<Box<dyn SourcePlugin>>>>,
	server: MyServer,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:53000".parse().unwrap();
	let server = MyServer::default();

	let runner_server = server.clone();
	task::spawn(async move {
		let server = runner_server;
		let mut plugins: Vec<Box<dyn SourcePlugin>> = vec![];
		plugins.push(
			tf_plugin_soundcloud::Soundcloud::new()
				.unwrap()
				.get_source_plugin()
				.unwrap(),
		);
		plugins.push(
			tf_plugin_youtube::Youtube::new()
				.unwrap()
				.get_source_plugin()
				.unwrap(),
		);

		let mut interval = time::interval(Duration::from_millis(1000));

		loop {
			interval.tick().await;
			let mut booth = server.booth.write().unwrap();
			let mut users = server.users.write().unwrap();
			if let Some(booth) = booth.as_mut() {
				if booth.started_at.elapsed() >= booth.duration {
					for _ in 0..50 {
						booth.user_queue.push_back(booth.dj);
						let next_dj = booth.user_queue.pop_front().unwrap();
						booth.dj = next_dj;
						let dj_queue = users.get_mut(&next_dj).unwrap().queue.as_mut().unwrap();
						let next_track = dj_queue.pop_front().unwrap();
						booth.track = next_track.clone();

						match next_track
							.url
							.parse::<Url>()
							.map_err(|e| anyhow!("invalid url: {e}"))
							.and_then(|url| {
								plugins
									.iter()
									.filter_map(|p| {
										p.handle_url(&url).map(|p| {
											p.map_err(|e| {
												anyhow!("failed to handle this track: {e}")
											})
										})
									})
									.next()
									.ok_or(anyhow!("no plugin can handle this track"))
							}) {
							Ok(_) => {
								dj_queue.push_back(next_track.clone());
							}
							Err(e) => {
								println!("{e}");
								if dj_queue.len() == 0 {
									panic!("oops no tracks");
								}
							}
						}
					}
				}
			}
		}
	});

	Server::builder()
		.add_service(HubdjServer::new(server))
		.serve(addr)
		.await?;

	Ok(())
}
