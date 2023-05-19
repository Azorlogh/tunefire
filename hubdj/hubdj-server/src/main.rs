mod probe;

use std::{
	collections::{HashMap, VecDeque},
	pin::Pin,
	sync::Arc,
	time::{Duration, Instant},
};

use anyhow::{anyhow, Context};
use hubdj_core::{UserId, UserToken};
use parking_lot::RwLock;
use tf_plugin::{Plugin, SourcePlugin};
use tokio::{sync::mpsc, task, time};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{codegen::futures_core::Stream, transport::Server, Request, Response, Status};

pub mod pb {
	tonic::include_proto!("hubdj");
}

use pb::{
	hubdj_server::HubdjServer, AuthRequest, AuthResponse, JoinQueueRequest, LeaveQueueRequest,
	QueuedTrack, SubmitPlaylistRequest,
};
use url::Url;

type HResult<T> = Result<Response<T>, Status>;

#[derive(Debug)]
pub struct User {
	id: UserId,
	token: UserToken,
	name: String,
	queue: VecDeque<pb::Track>,
}

#[derive(Debug)]
pub struct CurrentDj {
	user: UserId,
	track: pb::Track,
	started_at: Instant,
	duration: Duration,
}

#[derive(Debug, Default)]
pub struct Booth {
	current_dj: Option<CurrentDj>,
	user_queue: VecDeque<UserId>,
}

#[derive(Clone)]
pub struct MyServer {
	users: Arc<RwLock<HashMap<UserId, User>>>,
	booth: Arc<RwLock<Booth>>,
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

fn get_booth_state(users: &HashMap<UserId, User>, booth: &Booth) -> Option<pb::booth::Playing> {
	if let Some(dj) = &booth.current_dj {
		Some(pb::booth::Playing {
			dj: dj.user.0,
			track: Some(dj.track.clone()),
			elapsed: dj.started_at.elapsed().as_millis() as u64,
			queue: booth
				.user_queue
				.iter()
				.map(|uid| QueuedTrack {
					track: Some(users.get(uid).unwrap().queue.front().unwrap().clone()),
					user: Some(pb::UserId { id: uid.0 }),
				})
				.collect(),
		})
	} else {
		None
	}
}

impl MyServer {
	pub async fn send_booth_state(&self) -> Result<(), Box<dyn std::error::Error>> {
		let booth = pb::Booth {
			playing: get_booth_state(&self.users.read(), &self.booth.read()),
		};
		let listeners = self.booth_listeners.read().clone();
		for sender in listeners {
			sender.send(Result::Ok(booth.clone())).await?
		}
		Ok(())
	}
}

#[tonic::async_trait]
impl pb::hubdj_server::Hubdj for MyServer {
	async fn auth(&self, request: Request<AuthRequest>) -> HResult<AuthResponse> {
		let (id, token) = (UserId(rand::random()), UserToken(rand::random()));

		let mut users = self.users.write();

		users.insert(
			id,
			User {
				id,
				token,
				name: request.into_inner().name,
				queue: VecDeque::new(),
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
		let users = self.users.read();
		let user = users
			.get(&id)
			.ok_or(Status::not_found("user does not exist"))?;
		Ok(Response::new(pb::User {
			id: id.0,
			name: user.name.clone(),
			queue: Some(pb::user::Queue {
				tracks: Vec::from(user.queue.clone()),
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
		self.booth_listeners.write().push(tx.clone());
		tokio::spawn(async move {
			let booth = {
				let users = users.read();
				pb::Booth {
					playing: get_booth_state(&users, &booth.read()),
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
		let mut users = self.users.write();
		let user = users
			.values_mut()
			.find(|u| u.token == UserToken(args.token));
		if let Some(user) = user {
			user.queue = VecDeque::from_iter(args.playlist.unwrap().tracks.into_iter());
		}
		Ok(Response::new(pb::Status { ok: true }))
	}

	async fn join_queue(&self, request: Request<JoinQueueRequest>) -> HResult<pb::Status> {
		let token = request.into_inner().token;
		let success = {
			let users = self.users.read();
			let user = users.values().find(|u| u.token == UserToken(token));
			if let Some(user) = user {
				let mut booth = self.booth.write();
				booth.user_queue.push_back(user.id);
				true
			} else {
				false
			}
		};
		// if success {
		// 	self.send_booth_state().await.unwrap();
		// }
		Ok(Response::new(pb::Status { ok: success }))
	}

	async fn leave_queue(&self, request: Request<LeaveQueueRequest>) -> HResult<pb::Status> {
		let token = request.into_inner().token;
		let users = self.users.read();
		let user = users.values().find(|u| u.token == UserToken(token));
		if let Some(user) = user {
			let mut booth = self.booth.write();
			booth.user_queue.retain(|u| *u != user.id);
		}
		Ok(Response::new(pb::Status { ok: true }))
	}
}

// pub struct Runner {
// 	plugins: Arc<RwLock<Vec<Box<dyn SourcePlugin>>>>,
// 	server: MyServer,
// }

fn handle_track(
	plugins: &[Box<dyn tf_plugin::SourcePlugin>],
	track: &pb::Track,
) -> anyhow::Result<tf_plugin::player::TrackInfo> {
	let url = track
		.url
		.parse::<Url>()
		.map_err(|e| anyhow!("invalid url: {e}"))?;
	let track = plugins
		.iter()
		.filter_map(|p| {
			p.handle_url(&url)
				.map(|p| p.map_err(|e| anyhow!("failed to handle this track: {e}")))
		})
		.next()
		.ok_or(anyhow!("no plugin can handle this track"))
		.and_then(|r| r.context("failed to handle this track"))?;
	Ok(track.info)
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
			let changed = {
				let mut changed = false;
				let mut booth = server.booth.write();
				let mut users = server.users.write();
				// println!("{booth:#?} {users:#?}");
				if let Some(dj) = booth.current_dj.take() {
					if dj.started_at.elapsed() >= dj.duration {
						booth.user_queue.push_back(dj.user);
						booth.current_dj = None;
						changed = true;
					}
					booth.current_dj = Some(dj);
				}
				while booth.current_dj.is_none() && booth.user_queue.len() > 0 {
					changed = true;
					let next_dj = booth.user_queue.pop_front().unwrap();
					let dj_queue = &mut users.get_mut(&next_dj).unwrap().queue;
					if let Some(next_track) = dj_queue.pop_front() {
						match handle_track(&plugins, &next_track) {
							Ok(track) => {
								let new_dj = CurrentDj {
									user: next_dj,
									track: next_track.clone(),
									started_at: Instant::now(),
									duration: track.duration,
								};
								booth.current_dj = Some(new_dj);
								dj_queue.push_back(next_track.clone());
							}
							Err(e) => {
								println!("{e}");
							}
						}
					} else {
						println!("dj had no tracks");
						continue;
					}
				}
				changed
			};
			if changed {
				server.send_booth_state().await.ok(); // TODO: remove disconnected listeners
			}
		}
	});

	Server::builder()
		.add_service(HubdjServer::new(server))
		.serve(addr)
		.await?;

	Ok(())
}
