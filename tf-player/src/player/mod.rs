use std::{
	collections::VecDeque,
	ops::DerefMut,
	sync::{
		atomic::{self, AtomicUsize},
		Arc,
	},
	time::{Duration, Instant},
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	StreamConfig,
};
use parking_lot::RwLock;
use tracing::debug;

use crate::{PlayerController, SongSource, SourceError};

pub mod state;
pub use state::State;

mod resampler;
use resampler::Resampler;

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
	buffering_since: Instant,
}

impl Player {
	pub fn new() -> anyhow::Result<PlayerController> {
		let host = cpal::default_host();

		let device = host
			.default_output_device()
			.expect("no output device available");

		let config = device.default_output_config().expect("error while configs");

		debug!("playing... {:?}", config);

		let (to_player, from_controller) = crossbeam_channel::unbounded();

		let nb_queued = Arc::new(AtomicUsize::new(0));
		let mut player = Player {
			receiver: from_controller,
			config: config.config(),
			state: Arc::new(RwLock::new(State::Idle)),
			source_queue: VecDeque::new(),
			nb_queued: nb_queued.clone(),
			source: None,
			resampler: None,
			buffering_since: Instant::now(),
		};

		let player_state = player.state.clone();

		let stream = device.build_output_stream(
			&config.into(),
			move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
				player.process(data);
			},
			move |err| {
				println!("{err:?}");
			},
		)?;

		stream.play().unwrap();

		PlayerController::new(player_state, to_player, stream, nb_queued)
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
				}
				Event::Pause => {
					self.state.write().pause().ok();
				}
				Event::Stop => {
					*self.state.write() = State::Idle;
				}
				Event::Seek(position) => {
					self.state.write().seek(position).unwrap();
					match self.source.as_mut().map(|s| s.signal.seek(position)) {
						Some(Err(SourceError::Buffering)) => {
							self.buffering_since = Instant::now();
						}
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

	pub fn process(&mut self, data: &mut [f32]) {
		self.process_events();

		self.next_source();

		let playing = match *self.state.read() {
			State::Playing(state::Playing { paused, .. }) => !paused,
			_ => false,
		};

		if !playing {
			for d in data {
				*d = 0.0;
			}
			return;
		}

		let resampler = self.resampler.as_mut().unwrap();
		let source = self.source.as_mut().unwrap();

		if self.buffering_since.elapsed() < Duration::from_secs(1) {
			for d in data {
				*d = 0.0;
			}
			return;
		}

		for d in data.chunks_mut(2) {
			if resampler.i >= resampler.out_buf[0].len() {
				match resampler.process(source) {
					Err(SourceError::Buffering) => {
						// we should probably zero out the rest of the buffer
						self.buffering_since = Instant::now();
						return;
					}
					Err(SourceError::EndOfStream) => {
						self.resampler = None;
						self.source = None;
						*self.state.write() = State::Idle;
						self.next_source();
						return;
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
			d[0] = resampler.out_buf[0][resampler.i];
			d[1] = resampler.out_buf[1][resampler.i];
			resampler.i += 1;
		}
	}
}
