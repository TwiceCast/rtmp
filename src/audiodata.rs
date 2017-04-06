use header::Header;
use std::io::{Read, Write};
use message;

pub struct AudioData {
  pub data: Vec<u8>,
}

impl AudioData {
	pub fn read<R: Read>(header: &mut Header, reader: &mut R) -> Result<AudioData, ()> {
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
    Ok(AudioData{data: slice})
	}

  pub fn write<W: Write>(&self, writer: &mut W) {
    writer.write(&self.data).unwrap();
  }
}

impl ::std::fmt::Debug for AudioData {
  fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    fmt.debug_struct("AudioData")
       .finish()
  }
}