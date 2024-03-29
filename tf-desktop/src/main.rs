use anyhow::{anyhow, Result};
use druid::{AppLauncher, WindowDesc};

#[macro_use]
mod util;

mod command;
mod state;
mod ui;
pub use state::State;
use tf_gui::data;
use tracing_subscriber::EnvFilter;

pub mod widget;

pub mod theme;

mod delegate;

mod media_controls;

mod controller;

fn main() -> Result<()> {
	use tracing_subscriber::prelude::*;
	let fmt_layer = tracing_subscriber::fmt::layer()
		.without_time()
		.with_target(true)
		.with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
			metadata.target().starts_with("tf");
			true
		}));
	tracing_subscriber::registry()
		.with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")))
		.with(fmt_layer)
		.init();

	let mut db = connect_to_db()?;

	let main_window = WindowDesc::new(ui::ui(&db)).window_size((1000.0, 800.0));
	let state = State::new(&mut db)?;
	AppLauncher::with_window(main_window)
		.delegate(delegate::Delegate::new(db)?)
		.configure_env(theme::apply)
		.launch(state)
		.map_err(|err| anyhow!("failed to start app: {}", err))
}

fn connect_to_db() -> Result<tf_db::Client> {
	let dirs = directories::ProjectDirs::from("", "Azorlogh", "tunefire")
		.expect("failed to get data directory");
	std::fs::create_dir_all(dirs.data_dir())?;
	let db_path = dirs.data_dir().join("db.slab");
	tf_db::Client::new(db_path)
}
