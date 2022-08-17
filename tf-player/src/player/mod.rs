use std::{
	collections::VecDeque,
	ops::DerefMut,
	sync::{
		atomic::{self, AtomicUsize},
		Arc,
	},
	time::Duration,
};

use cpal::{traits::StreamTrait, StreamConfig};
use parking_lot::RwLock;
use tracing::{debug, error};

use crate::{SongSource, SourceError};

pub mod sink;
pub mod state;
pub use state::State;

mod resampler;
use resampler::Resampler;

mod controller;
pub use controller::Controller;

pub enum Event {
	QueueSong(SongSource),
	Play,
	Pause,
	Stop,
	Seek(Duration),
}

pub struct Player {
	receiver: crossbeam_channel::Receiver<Event>,
	config: StreamConfig,
	state: Arc<RwLock<State>>,
	source_queue: VecDeque<SongSource>,
	nb_queued: Arc<AtomicUsize>, // invariant: nb_queued == source_queue.len()
	source: Option<SongSource>,
	resampler: Option<Resampler>,
	audio_sink: rtrb::Producer<f32>,
	stream: cpal::Stream,
}

impl Player {
	pub fn new() -> anyhow::Result<Controller> {
		let (to_player, from_controller) = crossbeam_channel::unbounded();

		let player_state = Arc::new(RwLock::new(State::Idle));

		let nb_queued = Arc::new(AtomicUsize::new(0));

		let decoder_player_state = player_state.clone();
		let decoder_nb_queued = nb_queued.clone();
		std::thread::Builder::new()
			.name("decoder".to_owned())
			.spawn(move || {
				let (stream, config, audio_sink) = sink::AudioSink::new().unwrap();
				debug!("launched decoder thread");
				let mut player = Player {
					receiver: from_controller,
					config,
					state: decoder_player_state,
					source_queue: VecDeque::new(),
					nb_queued: decoder_nb_queued,
					source: None,
					resampler: None,
					audio_sink,
					stream,
				};
				loop {
					player.process();
				}
			})
			.unwrap();

		Controller::new(player_state, to_player, nb_queued)
	}

	pub fn process_events(&mut self) {
		while let Ok(event) = self.receiver.try_recv() {
			match event {
				Event::QueueSong(source) => {
					self.source_queue.push_back(source);
					self.nb_queued
						.store(self.source_queue.len(), atomic::Ordering::Relaxed);
				}
				Event::Play => {
					self.state.write().play().ok();
					self.stream.play().unwrap();
				}
				Event::Pause => {
					self.state.write().pause().ok();
					self.stream.pause().unwrap();
				}
				Event::Stop => {
					self.stream.pause().unwrap();
					*self.state.write() = State::Idle;
				}
				Event::Seek(position) => {
					self.state.write().seek(position).unwrap();
					match self.source.as_mut().map(|s| s.signal.seek(position)) {
						Some(Err(e)) => error!("{e:?}"),
						_ => {}
					}
				}
			}
		}
	}

	pub fn next_source(&mut self) {
		if *self.state.read() != State::Idle {
			return;
		}
		if let Some(mut source) = self.source_queue.pop_front() {
			let source_sample_rate = source.sample_rate;
			let mut resampler =
				Resampler::new((self.config.sample_rate.0 as f64) / source_sample_rate).unwrap();
			resampler.process(&mut source).ok();
			self.resampler = Some(resampler);
			self.state.write().set_song(source.info.clone());
			self.source = Some(source);
			self.nb_queued
				.store(self.source_queue.len(), atomic::Ordering::Relaxed);
		}
	}

	pub fn process(&mut self) {
		self.process_events();

		self.next_source();

		let playing = match *self.state.read() {
			State::Playing(state::Playing { paused, .. }) => !paused,
			_ => false,
		};

		if !playing {
			// for d in data {
			// 	*d = 0.0;
			// }
			return;
		}

		let resampler = self.resampler.as_mut().unwrap();
		let source = self.source.as_mut().unwrap();

		let missing_data = self.audio_sink.slots();

		if missing_data > 512 {
			for _ in 0..(missing_data / 2) {
				if resampler.i >= resampler.out_buf[0].len() {
					match resampler.process(source) {
						Err(SourceError::EndOfStream) => {
							self.resampler = None;
							self.source = None;
							*self.state.write() = State::Idle;
							self.next_source();
							return;
						}
						Err(e) => {
							panic!("{e:?}");
						}
						Ok(()) => match self.state.write().deref_mut() {
							State::Playing(state::Playing { offset, .. }) => {
								*offset += Duration::from_secs_f64(
									resampler.out_buf[0].len() as f64
										/ self.config.sample_rate.0 as f64,
								);
							}
							_ => {}
						},
					}
				}
				self.audio_sink
					.push(resampler.out_buf[0][resampler.i])
					.unwrap();
				self.audio_sink
					.push(resampler.out_buf[1][resampler.i])
					.unwrap();
				resampler.i += 1;
			}
		}
	}
}
