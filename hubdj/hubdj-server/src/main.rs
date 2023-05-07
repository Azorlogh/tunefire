use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};

use tonic::{transport::Server, Request, Response, Status};

pub mod pb {
	tonic::include_proto!("hubdj");
}

use pb::{hubdj_server::HubdjServer, AuthRequest, AuthResponse, Playlist};
use url::Url;

type HResult<T> = Result<Response<T>, Status>;

pub struct Client {
	id: ClientId,
	token: ClientToken,
	name: String,
	playlist: Vec<pb::Track>,
}

#[derive(Default)]
pub struct MyServer {
	clients: Arc<RwLock<HashMap<ClientId, Client>>>,
}

#[tonic::async_trait]
impl pb::hubdj_server::Hubdj for MyServer {
	async fn auth(&self, request: Request<AuthRequest>) -> HResult<AuthResponse> {
		let (id, token) = (ClientId(rand::random()), ClientToken(rand::random()));

		let mut clients = self.clients.write().unwrap();

		clients.insert(
			id,
			Client {
				id,
				token,
				playlist: vec![],
				name: request.into_inner().name,
			},
		);

		let clients = clients.keys().map(|cid| cid.0).collect();
		let res = AuthResponse {
			token: token.0,
			id: id.0,
			clients,
		};
		Ok(Response::new(res))
	}

	async fn get_client(&self, request: Request<pb::ClientId>) -> HResult<pb::Client> {
		let id = ClientId(request.into_inner().id);
		let clients = self.clients.read().unwrap();
		let client = clients
			.get(&id)
			.ok_or(Status::not_found("client does not exist"))?;
		Ok(Response::new(pb::Client {
			id: id.0,
			name: client.name.clone(),
			tracks: client.playlist.clone(),
		}))
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
