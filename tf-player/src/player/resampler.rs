use anyhow::Result;
use rubato::{InterpolationParameters, Resampler as _, SincFixedOut};

use crate::{SourceError, TrackSource};

pub struct Resampler {
	pub resampler: SincFixedOut<f32>,
	pub source_buf: Vec<[f32; 2]>,
	pub in_buf: Vec<Vec<f32>>,
	pub out_buf: Vec<Vec<f32>>,
	pub i: usize,
}

impl Resampler {
	pub fn new(ratio: f64) -> Result<Self> {
		let resampler = SincFixedOut::new(
			ratio,
			2.0,
			InterpolationParameters {
				sinc_len: 256,
				f_cutoff: 0.95,
				oversampling_factor: 128,
				interpolation: rubato::InterpolationType::Linear,
				window: rubato::WindowFunction::Blackman2,
			},
			512,
			2,
		)
		.unwrap();

		Ok(Self {
			source_buf: vec![[0.0; 2]; resampler.input_frames_max()],
			in_buf: resampler.input_buffer_allocate(),
			out_buf: resampler.output_buffer_allocate(),
			resampler,
			i: 0,
		})
	}

	pub fn process(&mut self, source: &mut TrackSource) -> Result<(), SourceError> {
		let in_len = self.resampler.input_frames_next();
		source.signal.next(&mut self.source_buf[..in_len])?;

		self.in_buf[0].clear();
		self.in_buf[1].clear();
		for i in 0..in_len {
			self.in_buf[0].push(self.source_buf[i][0]);
			self.in_buf[1].push(self.source_buf[i][1]);
		}

		self.out_buf[0].clear();
		self.out_buf[1].clear();
		self.resampler
			.process_into_buffer(&self.in_buf, &mut self.out_buf, Some(&[true, true]))
			.unwrap();
		self.i = 0;
		Ok(())
	}
}
