use std::{fs::File, path::Path, time::Duration};

// use anyhow::{Context, Result};
use symphonia::core::{
	audio::{AudioBufferRef, SampleBuffer},
	codecs::Decoder,
	errors::Error as SymphoniaError,
	formats::{FormatReader, SeekMode, SeekTo},
	io::MediaSourceStream,
	probe::Hint,
	units::Time,
};

use crate::SourceError;

pub struct Source {
	format: Box<dyn FormatReader>,
	decoder: Box<dyn Decoder>,
	track_id: u32,
	pub duration: Duration,
	pub sample_rate: f64,
	sample_buf: symphonia::core::audio::SampleBuffer<f32>,
	i: u32,
}

impl Source {
	pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
		let path = path.as_ref();
		let file = Box::new(File::open(path).unwrap());
		let mss = MediaSourceStream::new(file, Default::default());
		let mut hint = Hint::new();
		if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
			hint.with_extension(extension);
		}
		Self::from_mss(mss, hint)
	}

	pub fn from_mss(mss: MediaSourceStream, hint: Hint) -> Result<Self, anyhow::Error> {
		let probed = symphonia::default::get_probe()
			.format(&hint, mss, &Default::default(), &Default::default())
			.unwrap();
		let format = probed.format;
		Self::from_format_reader(format)
	}

	pub fn from_format_reader(mut format: Box<dyn FormatReader>) -> Result<Self, anyhow::Error> {
		let track = format.default_track().unwrap().clone();
		let mut decoder =
			symphonia::default::get_codecs().make(&track.codec_params, &Default::default())?;
		let codec_params = decoder.codec_params();
		let duration = {
			let time = codec_params
				.time_base
				.unwrap()
				.calc_time(codec_params.n_frames.unwrap());
			Duration::from_secs(time.seconds) + Duration::from_secs_f64(time.frac)
		};
		let audio_buf = get_next_audio_buffer(&mut *format, track.id, &mut *decoder)?;
		let spec = *audio_buf.spec();
		let sample_buf = {
			let duration = audio_buf.capacity() as u64;
			let mut sample_buf = SampleBuffer::new(duration, spec);
			sample_buf.copy_interleaved_ref(audio_buf);
			sample_buf
		};
		Ok(Self {
			format,
			decoder,
			track_id: track.id,
			duration,
			sample_rate: spec.rate as f64,
			sample_buf,
			i: 0,
		})
	}

	fn decode_next(&mut self) -> Result<(), SourceError> {
		let next = get_next_audio_buffer(&mut *self.format, self.track_id, &mut *self.decoder);
		match next {
			Ok(audio_buf) => {
				self.sample_buf.copy_interleaved_ref(audio_buf);
				self.i = 0;
				Ok(())
			}
			Err(SymphoniaError::IoError(e)) => match e.kind() {
				std::io::ErrorKind::UnexpectedEof => Err(SourceError::EndOfStream),
				_ => Err(SourceError::General(Box::new(e))),
			},
			Err(e) => Err(SourceError::General(Box::new(e))),
		}
	}
}

fn get_next_audio_buffer<'a>(
	format: &mut dyn FormatReader,
	track_id: u32,
	decoder: &'a mut dyn Decoder,
) -> Result<AudioBufferRef<'a>, SymphoniaError> {
	let packet = loop {
		let packet = format.next_packet()?;
		if packet.track_id() == track_id {
			break packet;
		}
	};
	let audio_buf = decoder.decode(&packet)?;
	Ok(audio_buf)
}

impl crate::Source for Source {
	fn seek(&mut self, pos: Duration) -> Result<(), SourceError> {
		self.format
			.seek(
				SeekMode::Coarse,
				SeekTo::Time {
					time: Time {
						seconds: pos.as_secs(),
						frac: pos.as_secs_f64().fract(),
					},
					track_id: None,
				},
			)
			.map_err(|e| SourceError::General(Box::new(e)))?;
		self.decode_next()?;
		self.i = 0;
		Ok(())
	}

	fn next(&mut self, buf: &mut [[f32; 2]]) -> Result<(), SourceError> {
		for b in buf {
			if self.i >= self.sample_buf.len() as u32 {
				self.decode_next()?;
			}
			b[0] = self.sample_buf.samples()[(self.i + 0) as usize];
			b[1] = self.sample_buf.samples()[(self.i + 1) as usize];
			self.i += 2;
		}
		Ok(())
	}
}
