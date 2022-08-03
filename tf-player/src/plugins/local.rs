use anyhow::Result;
use url::Url;

use crate::{plugin::SongInfo, SongSource, SourcePlugin};

pub struct LocalPlugin;

impl SourcePlugin for LocalPlugin {
	fn name(&self) -> &'static str {
		"Local"
	}

	fn handle_url(&self, url: &Url) -> Option<Result<SongSource>> {
		if url.scheme() == "file" {
			if let Ok(path) = url.to_file_path() {
				let source =
					crate::util::symphonia::Source::from_file(path).map(|source| SongSource {
						info: SongInfo {
							duration: source.duration,
						},
						sample_rate: source.sample_rate,
						signal: Box::new(source),
					});
				Some(source)
			} else {
				None
			}
		} else {
			None
		}
	}
}

// pub struct LocalSource {
// 	format: Box<dyn FormatReader>,
// 	decoder: Box<dyn Decoder>,
// 	track_id: u32,
// 	pub duration: Duration,
// 	pub sample_rate: f64,
// 	sample_buf: symphonia::core::audio::SampleBuffer<f32>,
// 	i: u32,
// 	buffering: bool,
// }

// impl LocalSource {
// 	pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
// 		let file = Box::new(File::open(path).unwrap());
// 		let mss = MediaSourceStream::new(file, Default::default());
// 		Self::from_mss(mss)
// 	}

// 	pub fn from_mss(mss: MediaSourceStream) -> Result<Self> {
// 		let hint = Hint::new();

// 		let probed = symphonia::default::get_probe()
// 			.format(&hint, mss, &Default::default(), &Default::default())
// 			.unwrap();

// 		let mut format = probed.format;

// 		let track = format.default_track().unwrap().clone();

// 		let mut decoder =
// 			symphonia::default::get_codecs().make(&track.codec_params, &Default::default())?;
// 		let codec_params = decoder.codec_params();
// 		let duration = {
// 			let time = codec_params
// 				.time_base
// 				.unwrap()
// 				.calc_time(codec_params.n_frames.unwrap());
// 			Duration::from_secs(time.seconds) + Duration::from_secs_f64(time.frac)
// 		};

// 		let audio_buf = get_next_audio_buffer(&mut *format, track.id, &mut *decoder)?;
// 		let spec = *audio_buf.spec();
// 		let sample_buf = {
// 			let duration = audio_buf.capacity() as u64;
// 			let mut sample_buf = SampleBuffer::new(duration, spec);
// 			sample_buf.copy_interleaved_ref(audio_buf);
// 			sample_buf
// 		};

// 		Ok(Self {
// 			format,
// 			decoder,
// 			track_id: track.id,
// 			duration,
// 			sample_rate: spec.rate as f64,
// 			sample_buf,
// 			buffering: false,
// 			i: 0,
// 		})
// 	}

// 	fn decode_next(&mut self) {
// 		let next = get_next_audio_buffer(&mut *self.format, self.track_id, &mut *self.decoder);
// 		match next {
// 			Ok(audio_buf) => {
// 				self.sample_buf.copy_interleaved_ref(audio_buf);
// 				self.buffering = false;
// 			}
// 			Err(e) => {
// 				self.sample_buf.clear();
// 				self.buffering = true;
// 				println!("error while decoding next packed {}", e);
// 			}
// 		}
// 		self.i = 0;
// 	}
// }

// fn get_next_audio_buffer<'a>(
// 	format: &mut dyn FormatReader,
// 	track_id: u32,
// 	decoder: &'a mut dyn Decoder,
// ) -> Result<AudioBufferRef<'a>> {
// 	let packet = loop {
// 		let packet = format.next_packet().context("Couldn't fetch next packet")?;
// 		if packet.track_id() == track_id {
// 			break packet;
// 		}
// 	};
// 	let audio_buf = decoder
// 		.decode(&packet)
// 		.context("Couldn't decode next packet")?;
// 	Ok(audio_buf)
// }

// impl Source for LocalSource {
// 	fn seek(&mut self, pos: Duration) {
// 		if let Err(_) = self.format.seek(
// 			SeekMode::Coarse,
// 			SeekTo::Time {
// 				time: Time {
// 					seconds: pos.as_secs(),
// 					frac: pos.as_secs_f64().fract(),
// 				},
// 				track_id: None,
// 			},
// 		) {
// 			self.buffering = true;
// 		}
// 		self.decode_next();
// 		self.i = 0;
// 	}

// 	fn next(&mut self, buf: &mut [[f32; 2]]) -> Result<(), SourceError> {
// 		for b in buf {
// 			if self.i >= self.sample_buf.len() as u32 {
// 				self.decode_next();
// 				if self.buffering {
// 					return Err(SourceError::Buffering);
// 				}
// 			}
// 			b[0] = self.sample_buf.samples()[(self.i + 0) as usize];
// 			b[1] = self.sample_buf.samples()[(self.i + 1) as usize];
// 			self.i += 2;
// 		}
// 		Ok(())
// 	}
// }
