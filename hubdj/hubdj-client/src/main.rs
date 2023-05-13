mod controllers;
mod state;
#[macro_use]
mod util;
mod ui;

use anyhow::Result;
use controllers::client::ClientController;
use druid::{AppLauncher, WindowDesc};
use state::{State, StateDisconnected};

pub mod pb {
	tonic::include_proto!("hubdj");
}

fn connect_to_db() -> Result<tf_db::Client> {
	let dirs = directories::ProjectDirs::from("", "Azorlogh", "tunefire")
		.expect("failed to get data directory");
	std::fs::create_dir_all(dirs.data_dir())?;
	let db_path = dirs.data_dir().join("db.slab");
	tf_db::Client::new(db_path)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let controller = ClientController::new().await?;

	let db = connect_to_db()?;

	let state = State::Disconnected(StateDisconnected {
		name: String::from(""),
	});

	AppLauncher::with_window(WindowDesc::new(ui::ui(&db, controller)))
		.log_to_console()
		.configure_env(|env, _| tf_gui::theme::apply(env))
		.launch(state)
		.unwrap();

	Ok(())
}
