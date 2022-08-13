use std::{cmp::Ordering, io::Read};

use anyhow::Result;

pub struct Cache {
	buffer: Vec<u8>,
	segments: Vec<(usize, usize)>,
	curr_segment: (usize, usize),
	next_segment_idx: usize,
}

impl Cache {
	pub fn new(capacity: usize) -> Self {
		Self {
			buffer: vec![0; capacity],
			segments: vec![],
			curr_segment: (0, 0),
			next_segment_idx: 0,
		}
	}

	pub fn available_from<'a>(&mut self, from: usize) -> &mut [u8] {
		let until = self.curr_segment.1;
		&mut self.buffer[from..until]
	}

	pub fn fill(&mut self, reader: &mut impl Read) -> Result<()> {
		let start = self.curr_segment.1;
		println!("am I blocking ?");
		let n_read = reader.read(&mut self.buffer[start..])?;
		println!("maybe");
		self.curr_segment.1 += n_read;
		loop {
			if let Some(next_segment) = self.segments.get(self.next_segment_idx) {
				if self.curr_segment.1 >= next_segment.0 {
					self.curr_segment.1 = self.curr_segment.1.max(next_segment.1);
					self.segments.remove(self.next_segment_idx);
				} else {
					break;
				}
			} else {
				break;
			}
		}
		Ok(())
	}

	fn find_segment(&mut self, pos: usize) -> Result<usize, usize> {
		self.segments.binary_search_by(|s| {
			if pos >= s.0 && pos <= s.1 {
				Ordering::Equal
			} else if pos <= s.0 {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		})
	}

	// submit the current segment and initiate a new one
	// if the new one starts in a stored segment, remove that segment from the list and add its length to the current segment's end
	pub fn seek(&mut self, pos: usize) {
		self.segments
			.insert(self.next_segment_idx, self.curr_segment);

		match self.find_segment(pos) {
			Ok(overlapping_idx) => {
				let overlapping_segment = self.segments.remove(overlapping_idx);
				self.curr_segment = (overlapping_segment.0, overlapping_segment.1);
			}
			Err(next_idx) => {
				self.curr_segment = (pos, pos);
				self.next_segment_idx = next_idx;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn simple() {
		let mut cache = Cache::new(100);
		cache.fill(&mut &b"01234"[..]).unwrap();
		assert!(cache.segments.is_empty());
		cache.seek(10);
		cache.fill(&mut &b"01234"[..]).unwrap();
		assert_eq!(cache.segments, vec![(0, 5)]);
		cache.seek(6);
		assert_eq!(cache.segments, vec![(0, 5), (10, 15)]);
		cache.fill(&mut &b"01234"[..]).unwrap();
		assert_eq!(cache.segments, vec![(0, 5)]);
		assert_eq!(cache.curr_segment, (6, 15));
	}
}
