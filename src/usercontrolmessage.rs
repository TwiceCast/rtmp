use std::fmt;
use std::io::{Write, Read};
use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};

use header::Header;

#[derive(Debug)]
pub enum UserControlMessage {
	StreamBegin(u32),
	PingRequest(u32),
	PingResponse(u32),
}

impl UserControlMessage {

	pub fn read<R: Read>(reader: &mut R) -> Result<Self, ()> {
		let id = reader.read_u16::<BigEndian>().unwrap();
		match id {
			0 => {
				let stream_id = reader.read_u32::<BigEndian>().unwrap();
				return Ok(UserControlMessage::StreamBegin(stream_id))
			},
			6 =>  {
				let timestamp = reader.read_u32::<BigEndian>().unwrap();
				Ok(UserControlMessage::PingRequest(timestamp))
			},
			7 => {
				let timestamp = reader.read_u32::<BigEndian>().unwrap();
				Ok(UserControlMessage::PingResponse(timestamp))
			},
			_ => Err(())
		}
  	}

	pub fn send<W: Write>(&self, writer: &mut W) -> Result<(), ()> {
		let h = Header::new(2, 0, 6, 4, 0);
		h.write(writer).unwrap();
		match *self {
			UserControlMessage::StreamBegin(stream_id) => {
				writer.write_u16::<BigEndian>(0).unwrap();
				writer.write_u32::<BigEndian>(stream_id).unwrap();				
				Ok(())
			},
			UserControlMessage::PingRequest(timestamp) => {
				writer.write_u16::<BigEndian>(6).unwrap();
				writer.write_u32::<BigEndian>(timestamp).unwrap();				
				Ok(())
			},
			UserControlMessage::PingResponse(timestamp) => {
				writer.write_u16::<BigEndian>(7).unwrap();
				writer.write_u32::<BigEndian>(timestamp).unwrap();				
				Ok(())
			},
		}
	}

	pub fn write<W: Write>(&self, writer: &mut W) {
		match *self {
			UserControlMessage::StreamBegin(stream_id) => {
				writer.write_u16::<BigEndian>(0).unwrap();
				writer.write_u32::<BigEndian>(stream_id).unwrap();				
			},
			UserControlMessage::PingRequest(timestamp) => {
				writer.write_u16::<BigEndian>(6).unwrap();
				writer.write_u32::<BigEndian>(timestamp).unwrap();				
			},
			UserControlMessage::PingResponse(timestamp) => {
				writer.write_u16::<BigEndian>(7).unwrap();
				writer.write_u32::<BigEndian>(timestamp).unwrap();				
			},
		}
	}
}

impl fmt::Display for UserControlMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
        	UserControlMessage::StreamBegin(nb) => write!(f, "Stream number {} begin", nb),
        	UserControlMessage::PingRequest(timestamp) => write!(f, "Ping request at timestamp {}", timestamp),
        	UserControlMessage::PingResponse(timestamp) => write!(f, "Ping response at timestamp {}", timestamp),
        }
    }
}