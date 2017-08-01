use header::Header;
use std::io::{Read, Write};

use messageinterface::MessageInterface;

pub struct AudioData {
  pub data: Vec<u8>,
}

impl MessageInterface for AudioData {
  type Message = AudioData;

	fn read<R: Read>(header: Header, reader: &mut R) -> Result<Self::Message, ()> {
		let mut slice = vec![];
    let size = header.message_length;
		slice.resize(size as usize, 0);
    reader.read(&mut slice).unwrap();
    Ok(AudioData{data: slice})
	}

  fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
    let result = writer.write(&self.data).unwrap();
    Ok(result)
  }

  fn fill_header(&self, _header: &mut Header) {
  }
}

impl ::std::fmt::Debug for AudioData {
  fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    fmt.debug_struct("AudioData")
       .finish()
  }
}