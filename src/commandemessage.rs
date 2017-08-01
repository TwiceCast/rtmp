use std::fmt;
use std::io::{Read, Write};
use amf::{self, Value, Serialize};

use header::Header;
use messageinterface::MessageInterface;
use slicereader::SliceReader;

pub struct NetConnection {
  pub command_name: String,
  pub transaction_id: f64,
  pub properties: Value,
  pub opt: Option<Value>,
  vec: Vec<u8>,
}

impl NetConnection {
  pub fn new(command_name: String, transaction_id: f64, properties: Value, opt: Option<Value>) -> Self {
    let vec = Vec::new();
    let mut ret = NetConnection{command_name: command_name, transaction_id: transaction_id, properties: properties, opt: opt, vec: vec};
    let command_name = Value::String(ret.command_name.to_string());
    Self::serialize(&mut ret.vec, &command_name);
    let transaction_id = Value::Number(ret.transaction_id);
    Self::serialize(&mut ret.vec, &transaction_id);
    Self::serialize(&mut ret.vec, &ret.properties);
    match ret.opt {
      Some(ref opt) => {
        Self::serialize(&mut ret.vec, &opt);
      },
      None => (),
    }
    ret
  }

  fn serialize<W: Write>(writer: &mut W, v: & Value)
  {
    let ser = amf::Serializer::new(writer);
    v.serialize(ser).unwrap();
  }
}

impl MessageInterface for NetConnection {
  type Message = NetConnection;

  fn read<R: Read>(header: Header, reader: &mut R) -> Result<Self::Message, ()> {
    let mut slice = vec![];
    slice.resize(header.message_length as usize, 0);
    reader.read(&mut slice).unwrap();
    let mut de = amf::Deserializer::new_from_slice(&slice);
    let mut ret = NetConnection::new("".to_string(), 0., Value::Null, None);
    match amf::Deserialize::deserialize(&mut de).unwrap() {
      Value::String(s) => ret.command_name = s,
      _ => return Err(())
    }
    match amf::Deserialize::deserialize(&mut de).unwrap() {
      Value::Number(nb) => ret.transaction_id = nb,
      _ => return Err(())
    }
    match amf::Deserialize::deserialize(&mut de).unwrap() {
      Value::Object(map) => ret.properties = Value::Object(map),
      Value::Null => (),
      t => { println!("{}", t); return Err(()) },
    }
    match amf::Deserialize::deserialize(&mut de) {
      Ok(v) => ret.opt = Some(v),
      Err(amf::Error::UnexpectedEOF) => (),
      _ => return Err(())
    }
    debug!("Reception of {:?}", ret);
    Ok(ret)
  }

/*  pub fn send<W: Write>(&self, writer: &mut W) -> Result<(), ()> {
    let mut vec = Vec::new();
    let command_name = Value::String(self.command_name.to_string());
    Self::serialize(&mut vec, &command_name);
    let transaction_id = Value::Number(self.transaction_id);
    Self::serialize(&mut vec, &transaction_id);
    Self::serialize(&mut vec, &self.properties);
    match self.opt {
      Some(ref opt) => {
        Self::serialize(&mut vec, &opt);
      },
      None => (),
    }
    let h = Header::new(3, 0, vec.len() as u32, 20, 0);
    h.write(writer).unwrap();
    writer.write(&vec).unwrap();
    Ok(())
  }*/

  fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
    let result = writer.write(&self.vec).unwrap();
    debug!("Send of {:?}", self);
    trace!("data: {:?}", self.vec);    
    Ok(result)
  }

  fn fill_header(&self, header: &mut Header) {
    header.copy(Header::new(3, 0, self.vec.len() as u32, 20, 0));
  }
}

impl ::std::fmt::Debug for NetConnection {
  fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    fmt.debug_struct("NetConnection")
    .field("command_name", &self.command_name)
    .field("transaction_id", &self.transaction_id)
    .field("properties", &self.properties)
    .field("opt", &self.opt)
    .finish()
  }
}

pub struct NetStreamCommand {
  pub command_name: String,
  pub transaction_id: f64,
  pub properties: Value,
  pub opt: Vec<Value>,
}

impl NetStreamCommand {
  fn serialize<W: Write>(writer: &mut W, v: & Value)
  {
    let ser = amf::Serializer::new(writer);
    v.serialize(ser).unwrap();
  }
}

impl MessageInterface for NetStreamCommand {
  type Message = NetStreamCommand;

  fn read<R: Read>(header: Header, reader: &mut R) -> Result<Self::Message, ()> {
    let mut slice = vec![];
    slice.resize(header.message_length as usize, 0);
    reader.read(&mut slice).unwrap();
    let mut de = amf::Deserializer::new_from_slice(&slice);
    let mut ret = NetStreamCommand {command_name: "".to_string(), transaction_id: 0., properties: Value::Null, opt: Vec::new()};
    match amf::Deserialize::deserialize(&mut de).unwrap() {
      Value::String(s) => ret.command_name = s,
      _ => return Err(())
    }
    match amf::Deserialize::deserialize(&mut de).unwrap() {
      Value::Number(nb) => ret.transaction_id = nb,
      _ => return Err(())
    }
    match amf::Deserialize::deserialize(&mut de).unwrap() {
      Value::Object(map) => ret.properties = Value::Object(map),
      Value::Null => (),
      t => { println!("{}", t); return Err(()) },
    }
    loop {
      match amf::Deserialize::deserialize(&mut de) {
        Ok(v) => ret.opt.push(v),
        Err(amf::Error::UnexpectedEOF) => break,
        _ => return Err(())
      }      
    }
    debug!("Reception of {:?}", ret);
    Ok(ret)
  }

  fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
    let mut vec = Vec::new();
    let command_name = Value::String(self.command_name.to_string());
    Self::serialize(&mut vec, &command_name);
    let transaction_id = Value::Number(self.transaction_id);
    Self::serialize(&mut vec, &transaction_id);
    Self::serialize(&mut vec, &self.properties);
    for ref obj in &self.opt {
      Self::serialize(&mut vec, &obj);
    }
    let result = writer.write(&vec).unwrap();
    Ok(result)
  }

  fn fill_header(&self, _header: &mut Header) {
  }
}

impl ::std::fmt::Debug for NetStreamCommand {
  fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    fmt.debug_struct("NetConnection")
    .field("command_name", &self.command_name)
    .field("transaction_id", &self.transaction_id)
    .field("properties", &self.properties)
    .field("opt", &self.opt)
    .finish()
  }
}

#[derive(Debug)]
pub enum CommandeMessage {
	NetConnection(NetConnection),
  NetStreamCommand(NetStreamCommand),
}

impl MessageInterface for CommandeMessage {
  type Message = CommandeMessage;

	fn read<R: Read>(header: Header, reader: &mut R) -> Result<Self::Message, ()> {
    let mut slice = vec![];
    slice.resize(header.message_length as usize, 0);
    reader.read(&mut slice).unwrap();
    let mut de = amf::Deserializer::new_from_slice(&slice);
    let command_name;
    match amf::Deserialize::deserialize(&mut de).unwrap() {
      Value::String(s) => command_name = s,
      v => { println!("{:?}", v); return Err(())}
    }
    let mut slice_reader = SliceReader::new(&slice);
    match command_name.as_ref() {
      "connect" | "call" | "close" | "createStream" => Ok(CommandeMessage::NetConnection(NetConnection::read(header, &mut slice_reader).unwrap())),
      _ => Ok(CommandeMessage::NetStreamCommand(NetStreamCommand::read(header, &mut slice_reader).unwrap()))
    }
  }

  fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
    match *self {
      CommandeMessage::NetConnection(ref m) => m.send(writer),
      CommandeMessage::NetStreamCommand(ref m) => m.send(writer),
    }
  }

  fn fill_header(&self, header: &mut Header) {
    match *self {
      CommandeMessage::NetConnection(ref m) => m.fill_header(header),
      CommandeMessage::NetStreamCommand(ref m) => m.fill_header(header),
    }
  }
}

impl fmt::Display for CommandeMessage {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      CommandeMessage::NetConnection(ref mess) => write!(f, "{:?}", mess),
      CommandeMessage::NetStreamCommand(ref mess) => write!(f, "{:?}", mess),
    }
  }
}