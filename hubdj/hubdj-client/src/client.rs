pub struct Client {
	client: HubdjClient,
}

impl Client {
	pub fn new() -> Self {
		let mut client = HubdjClient::connect("http://[::1]:53000").await?;
	}

	pub fn auth(name: String) -> ClientId {
		let response = self.client.auth(AuthRequest { name: args.name }).await?;
	}
}
