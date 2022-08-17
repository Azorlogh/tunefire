use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream, StreamConfig,
};
use rtrb::{Consumer, Producer};
use tracing::debug;

pub struct AudioSink {
	consumer: Consumer<f32>,
	filled_with_zeros: bool,
}

impl AudioSink {
	pub fn new() -> anyhow::Result<(Stream, StreamConfig, Producer<f32>)> {
		let host = cpal::default_host();

		let device = host
			.default_output_device()
			.expect("no output device available");

		let config = device
			.default_output_config()
			.expect("error while configs")
			.config();

		let (producer, consumer) = rtrb::RingBuffer::new(44100);

		let mut sink = AudioSink {
			consumer,
			filled_with_zeros: false,
		};

		let stream = device.build_output_stream(
			&config.clone(),
			move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
				sink.process(data);
			},
			move |err| {
				println!("{err:?}");
			},
		)?;

		debug!("created stream for the audio driver... {:?}", config);

		stream.pause().unwrap();

		Ok((stream, config, producer))
	}

	pub fn process(&mut self, data: &mut [f32]) {
		if self.consumer.slots() < data.len() {
			if !self.filled_with_zeros {
				data.fill(0.0);
				self.filled_with_zeros = true;
			}
			return;
		}
		let chunk = self.consumer.read_chunk(data.len()).unwrap(); // available >= data.len()
		let (first, second) = chunk.as_slices();
		data[..first.len()].copy_from_slice(first);
		data[first.len()..].copy_from_slice(second);
		chunk.commit_all();
		self.filled_with_zeros = false;
	}
}
