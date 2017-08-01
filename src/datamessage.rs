use std::io::{Read, Write};
use amf::{Value, self, Serialize};

use header::Header;
use messageinterface::MessageInterface;

pub struct DataMessage {
  pub data: Vec<Value>,
}

impl DataMessage {
  fn serialize<W: Write>(writer: &mut W, v: & Value)
  {
    let ser = amf::Serializer::new(writer);
    v.serialize(ser).unwrap();
  }
}

impl MessageInterface for DataMessage {
  type Message = DataMessage;

	fn read<R: Read>(header: Header, reader: &mut R) -> Result<Self::Message, ()> {
		let mut slice = vec![];
		slice.resize(header.message_length as usize, 0);
    reader.read(&mut slice).unwrap();
    let mut de = amf::Deserializer::new_from_slice(&slice);
    let mut datamessage = DataMessage{ data: Vec::new() };
    loop {
      match amf::Deserialize::deserialize(&mut de) {
        Ok(v) => datamessage.data.push(v),
        Err(amf::Error::UnexpectedEOF) => return Ok(datamessage),
        _ => return Err(()),
      }      
    }
  }

  fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
    let mut vec = Vec::new();
    for ref obj in &self.data {
      Self::serialize(&mut vec, &obj);   
    }
    let result = writer.write(&vec).unwrap();
    Ok(result)
  }

  fn fill_header(&self, _header: &mut Header) {
  }
}

impl ::std::fmt::Debug for DataMessage {
  fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    fmt.debug_struct("DataMessage")
    .field("data", &self.data)
    .finish()
  }
}