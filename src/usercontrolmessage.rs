use std::fmt;
use std::io::{Write, Read};
use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};

use header::Header;
use messageinterface::MessageInterface;

#[derive(Debug)]
pub enum UserControlMessage {
	StreamBegin(u32),
	SetBufferLength(u32, u32),
	PingRequest(u32),
	PingResponse(u32),
}

impl MessageInterface for UserControlMessage {

	type Message = UserControlMessage;

	fn read<R: Read>(_header: Header, reader: &mut R) -> Result<UserControlMessage, ()> {
		let id = reader.read_u16::<BigEndian>().unwrap();
		match id {
			0 => {
				let stream_id = reader.read_u32::<BigEndian>().unwrap();
				return Ok(UserControlMessage::StreamBegin(stream_id))
			},
			3 => {
				let stream_id = reader.read_u32::<BigEndian>().unwrap();
				let buffer_length = reader.read_u32::<BigEndian>().unwrap();
				return Ok(UserControlMessage::SetBufferLength(stream_id, buffer_length))
			}
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

	fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
		let mut result = 0;
		match *self {
			UserControlMessage::StreamBegin(stream_id) => {
				writer.write_u16::<BigEndian>(0).unwrap();
				result += 2;
				writer.write_u32::<BigEndian>(stream_id).unwrap();				
				result += 4;
			},
			UserControlMessage::SetBufferLength(stream_id, buffer_length) => {
				writer.write_u16::<BigEndian>(3).unwrap();
				result += 2;
				writer.write_u32::<BigEndian>(stream_id).unwrap();
				result += 4;
				writer.write_u32::<BigEndian>(buffer_length).unwrap();			
				result += 4;
			},
			UserControlMessage::PingRequest(timestamp) => {
				writer.write_u16::<BigEndian>(6).unwrap();
				result += 2;
				writer.write_u32::<BigEndian>(timestamp).unwrap();				
				result += 4;
			},
			UserControlMessage::PingResponse(timestamp) => {
				writer.write_u16::<BigEndian>(7).unwrap();
				result += 2;
				writer.write_u32::<BigEndian>(timestamp).unwrap();
				result += 4;
			},
		}
		Ok(result)
	}

	fn fill_header(&self, header: &mut Header) {
		header.copy(Header::new(2, 0, 6, 4, 0));
	}
}

impl fmt::Display for UserControlMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
        	UserControlMessage::StreamBegin(nb) => write!(f, "Stream number {} begin", nb),
        	UserControlMessage::SetBufferLength(nb, buffer_length) => write!(f, "Buffer length for stream {} of time {}", nb, buffer_length),
        	UserControlMessage::PingRequest(timestamp) => write!(f, "Ping request at timestamp {}", timestamp),
        	UserControlMessage::PingResponse(timestamp) => write!(f, "Ping response at timestamp {}", timestamp),
        }
    }
}