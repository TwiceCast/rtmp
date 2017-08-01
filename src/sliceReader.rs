use std::io::{Read, Error, ErrorKind};

pub struct SliceReader<'a>
{
	slice: &'a [u8]
}

impl<'a> SliceReader<'a> {
	pub fn new(slice: &'a [u8]) -> SliceReader {
		SliceReader{slice: slice}
	}
}

impl<'a> Read for SliceReader<'a>
{
	fn read(&mut self, buff: &mut[u8]) -> Result<usize, Error> {
		let size = buff.len();
		if size > self.slice.len() {
			return Err(Error::new(ErrorKind::UnexpectedEof, "failed to fill whole buffer"));
		}
		let slice = &self.slice[..size];
		self.slice = &self.slice[size..];
		buff.clone_from_slice(&slice);
		Ok(size)
	}
}