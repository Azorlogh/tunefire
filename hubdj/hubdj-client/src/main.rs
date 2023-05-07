use clap::Parser;
use tonic::{
	metadata::MetadataValue,
	transport::{Channel, Server},
	Request, Response, Status,
};

pub mod pb {
	tonic::include_proto!("hubdj");
}

use pb::{hubdj_client::HubdjClient, AuthRequest, AuthResponse, Playlist, Status as HubdjStatus};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
	#[arg(short, long)]
	name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();

	println!("{:?}", response);

	Ok(())
}
