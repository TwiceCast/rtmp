use header::Header;
use std::io::{Read, Write};
use message;

pub struct VideoData {
  pub data: Vec<u8>,
}

impl VideoData {
	pub fn read<R: Read>(header: &mut Header, reader: &mut R) -> Result<VideoData, ()> {
		let mut slice = vec![];
    let mut size = header.message_length;
    unsafe {
      if message::CHUNK_SIZE < size {
        header.message_length -= message::CHUNK_SIZE;
        size = message::CHUNK_SIZE
      }      
    }
		slice.resize(size as usize, 0);
    reader.read(&mut slice).unwrap();
    Ok(VideoData{data: slice})
	}

  pub fn write<W: Write>(&self, writer: &mut W) {
    writer.write(&self.data).unwrap();
  }
}

impl ::std::fmt::Debug for VideoData {
  fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    fmt.debug_struct("VideoData")
//       .field("data", &self.data)
       .finish()
  }
}