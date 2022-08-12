pub struct Fetcher {
	url: String,
	// length: usize,
	// position: usize,
	length: usize,
	position: usize,
	reader: Box<dyn Read + Send + Sync>,
	buffer: Arc<Mutex<Vec<u8>>>,
}

impl Fetcher {
	pub fn spawn() -> Result<Fetcher> {}
}
