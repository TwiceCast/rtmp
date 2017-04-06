use std::fmt;
use std::io::{Read, Write};
use amf::{self, Value, Serialize};

use header::Header;

pub struct NetConnection {
  pub command_name: String,
  pub transaction_id: f64,
  pub properties: Value,
  pub opt: Option<Value>,
}

impl NetConnection {
  fn read(slice: &[u8]) -> Result<Self, ()> {
    let mut de = amf::Deserializer::new_from_slice(&slice);
    let mut ret = NetConnection {command_name: "".to_string(), transaction_id: 0., properties: Value::Null, opt: None};
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
    Ok(ret)
  }

  fn serialize<W: Write>(writer: &mut W, v: & Value)
  {
    let ser = amf::Serializer::new(writer);
    v.serialize(ser).unwrap();
  }

  pub fn send<W: Write>(&self, writer: &mut W) -> Result<(), ()> {
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
  }

  pub fn write<W: Write>(&self, writer: &mut W) {
    let mut vec = Vec::new();
    let command_name = Value::String(self.command_name.to_string());
    Self::serialize(&mut vec, &command_name);
    let transaction_id = Value::Number(self.transaction_id);
    Self::serialize(&mut vec, &transaction_id);
    Self::serialize(&mut vec, &self.properties);
    writer.write(&vec).unwrap();
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
  fn read(slice: &[u8]) -> Result<Self, ()> {
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
        Err(amf::Error::UnexpectedEOF) => return Ok(ret),
        _ => return Err(())
      }      
    }
  }

  fn serialize<W: Write>(writer: &mut W, v: & Value)
  {
    let ser = amf::Serializer::new(writer);
    v.serialize(ser).unwrap();
  }

  pub fn write<W: Write>(&self, writer: &mut W) {
    let mut vec = Vec::new();
    let command_name = Value::String(self.command_name.to_string());
    Self::serialize(&mut vec, &command_name);
    let transaction_id = Value::Number(self.transaction_id);
    Self::serialize(&mut vec, &transaction_id);
    Self::serialize(&mut vec, &self.properties);
    for ref obj in &self.opt {
      Self::serialize(&mut vec, &obj);
    }
    writer.write(&vec).unwrap();
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

impl CommandeMessage {
	pub fn read<R: Read>(header: & Header, reader: &mut R) -> Result<CommandeMessage, ()> {
    let mut slice = vec![];
    slice.resize(header.message_length as usize, 0);
    reader.read(&mut slice).unwrap();
    let mut de = amf::Deserializer::new_from_slice(&slice);
    let command_name;
    match amf::Deserialize::deserialize(&mut de).unwrap() {
      Value::String(s) => command_name = s,
      _ => return Err(())
    }
    match command_name.as_ref() {
      "connect" | "call" | "close" | "createStream" => Ok(CommandeMessage::NetConnection(NetConnection::read(&slice).unwrap())),
      _ => Ok(CommandeMessage::NetStreamCommand(NetStreamCommand::read(&slice).unwrap()))
    }
  }

  pub fn write<W: Write>(&self, writer: &mut W) {
    match *self {
      CommandeMessage::NetConnection(ref m) => m.write(writer),
      CommandeMessage::NetStreamCommand(ref m) => m.write(writer),
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