use std::error::Error;

use hubdj_core::UserId;
use tonic::{transport::Channel, Status};

use crate::pb::{hubdj_client::HubdjClient, AuthRequest};

pub struct Client {
	client: HubdjClient<Channel>,
}

impl Client {
	pub async fn new() -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			client: HubdjClient::connect("http://[::1]:53000").await?,
		})
	}

	pub async fn auth(&self, name: String) -> Result<UserId, Status> {
		UserId(
			self.client
				.auth(AuthRequest { name })
				.await?
				.into_inner()
				.id,
		)
	}
}
