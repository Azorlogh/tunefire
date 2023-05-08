use std::{
	collections::HashMap,
	f32::consts::E,
	pin::Pin,
	sync::{Arc, RwLock},
	time::Duration,
};

use hubdj_core::{UserId, UserToken};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{codegen::futures_core::Stream, transport::Server, Request, Response, Status};

pub mod pb {
	tonic::include_proto!("hubdj");
}

use pb::{hubdj_server::HubdjServer, AuthRequest, AuthResponse, Playlist};

type HResult<T> = Result<Response<T>, Status>;

pub struct User {
	id: UserId,
	token: UserToken,
	name: String,
	queue: Option<Vec<pb::Track>>,
}

pub struct Booth {
	dj: UserId,
	track: pb::Track,
	queue: Vec<pb::Track>,
}

#[derive(Default)]
pub struct MyServer {
	users: Arc<RwLock<HashMap<UserId, User>>>,
	booth: Arc<RwLock<Option<Booth>>>,
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
			queue: user.queue.clone().map(|q| pb::user::Queue { tracks: q }),
		}))
	}

	type StreamBoothStateStream = Pin<Box<dyn Stream<Item = Result<pb::Booth, Status>> + Send>>;
	async fn stream_booth_state(
		&self,
		_request: Request<()>,
	) -> HResult<Self::StreamBoothStateStream> {
		let (tx, rx) = mpsc::channel(128);
		let users = self.users.clone();
		let booth = self.booth.clone();
		tokio::spawn(async move {
			loop {
				let booth = pb::Booth {
					playing: booth
						.read()
						.unwrap()
						.as_ref()
						.map(|booth| pb::booth::Playing {
							dj: booth.dj.0,
							track: Some(booth.track.clone()),
							queue: booth.queue.clone(),
						}),
				};
				if let Err(_) = tx.send(Result::Ok(booth)).await {
					break;
				}
				tokio::time::sleep(Duration::from_secs(1)).await;
			}
		});
		let output_stream = ReceiverStream::new(rx);
		Ok(Response::new(
			Box::pin(output_stream) as Self::StreamBoothStateStream
		))
	}

	async fn submit_playlist(&self, request: Request<Playlist>) -> HResult<pb::Status> {
		todo!()
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:53000".parse().unwrap();

	Server::builder()
		.add_service(HubdjServer::new(MyServer::default()))
		.serve(addr)
		.await?;

	Ok(())
}
