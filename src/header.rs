use std::io::{Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Header {
	pub chunk_stream_id: u32,
	pub timestamp: u32,
	delta_timestamp: u32,
	pub message_length: u32,
	pub message_type_id: u8,
	pub message_stream_id: u32,
	extend_timestamp: bool,
	htype: u8,
}

#[allow(dead_code)]
impl Header {

	pub fn new(chunk_stream_id: u32, timestamp: u32, message_length:u32, message_type_id: u8, message_stream_id: u32) -> Self {
		Header{
			chunk_stream_id: chunk_stream_id,
			timestamp: timestamp,
			delta_timestamp: 0,
			message_length: message_length,
			message_type_id: message_type_id,
			message_stream_id: message_stream_id,
			extend_timestamp: false,
			htype: 0,
		}
	}

	pub fn copy(&mut self, header: Header) {
		self.chunk_stream_id = header.chunk_stream_id;
		self.timestamp = header.timestamp;
		self.delta_timestamp = header.delta_timestamp;
		self.message_length = header.message_length;
		self.message_type_id = header.message_type_id;
		self.message_stream_id = header.message_stream_id;
		self.extend_timestamp = header.extend_timestamp;
		self.htype = header.htype;
	}

	fn read_header0<R: Read>(reader: &mut R, chunk_stream_id: u32, _last_header: Option<Header>) -> Result<Header, ()> {
		let mut timestamp = reader.read_int::<BigEndian>(3).unwrap() as u32;
		let message_length = reader.read_int::<BigEndian>(3).unwrap() as i64;
		let message_type_id = reader.read_u8().unwrap();
		let message_stream_id = reader.read_u32::<LittleEndian>().unwrap();
		let extend_timestamp;
		if timestamp >= 0xFFFFFF {
			//println!("extend timestamp {}",chunk_stream_id);
			extend_timestamp = true;
			timestamp = reader.read_u32::<BigEndian>().unwrap();
		} else {
			extend_timestamp = false;
		}
		Ok(Header{
			chunk_stream_id: chunk_stream_id,
			timestamp: timestamp,
			delta_timestamp: 0,
			message_length: message_length as u32,
			message_type_id: message_type_id,
			message_stream_id: message_stream_id,
			extend_timestamp: extend_timestamp,
			htype: 0,
		})
	}

	fn read_header1<R: Read>(reader: &mut R, chunk_stream_id: u32, last_header: Option<Header>) -> Result<Header, ()> {
		match last_header {
			None => Err(()),
			Some(h) => {
				let mut timestamp = reader.read_int::<BigEndian>(3).unwrap() as u32;
				let message_length = reader.read_int::<BigEndian>(3).unwrap() as i64;
				let message_type_id = reader.read_u8().unwrap();
				let extend_timestamp;
				//println!("{:?}, {:?}", h.timestamp, timestamp);
				if timestamp >= 0xFFFFFF {
					//println!("extend timestamp {}",chunk_stream_id);
					extend_timestamp = true;
					timestamp = reader.read_u32::<BigEndian>().unwrap();
				} else {
					extend_timestamp = false;
				}
				Ok(Header{
					chunk_stream_id: chunk_stream_id,
					timestamp: h.timestamp + timestamp,
					delta_timestamp: timestamp,
					message_length: message_length as u32,
					message_type_id: message_type_id,
					message_stream_id: h.message_stream_id,
					extend_timestamp: extend_timestamp,
					htype: 1,
				})
			}			
		}
	}

	fn read_header2<R: Read>(reader: &mut R, chunk_stream_id: u32, last_header: Option<Header>) -> Result<Header, ()> {
		match last_header {
			None => Err(()),
			Some(h) => {
				let timestamp = reader.read_int::<BigEndian>(3).unwrap() as u32;
				let extend_timestamp = false;
				Ok(Header{
					chunk_stream_id: chunk_stream_id,
					timestamp: h.timestamp + timestamp,
					delta_timestamp: timestamp,
					message_length: h.message_length,
					message_type_id: h.message_type_id,
					message_stream_id: h.message_stream_id,
					extend_timestamp: extend_timestamp,
					htype: 2,
				})
			}			
		}
	}

	fn read_header3<R: Read>(_reader: &mut R, chunk_stream_id: u32, last_header: Option<Header>) -> Result<Header, ()> {
		match last_header {
			None => Err(()),
			Some(h) => {
				let timestamp = h.timestamp + h.delta_timestamp;
				Ok(Header{
					chunk_stream_id: chunk_stream_id,
					timestamp: timestamp,
					delta_timestamp: h.delta_timestamp,
					message_length: h.message_length,
					message_type_id: h.message_type_id,
					message_stream_id: h.message_stream_id,
					extend_timestamp: h.extend_timestamp,
					htype: 3,
				})
			}			
		}
	}

	fn read_chunk_stream_id<R: Read>(reader: &mut R, chunk_stream_id: u8) -> Result<u32, ()> {
		match chunk_stream_id {
			0 => {
				let chunk_stream_id = reader.read_u8().unwrap() + 64;
				Ok(chunk_stream_id as u32)
			},
			1 => {
				let chunk_stream_id = reader.read_u16::<BigEndian>().unwrap() + 64;
				Ok(chunk_stream_id as u32)
			},
			_ => Ok(chunk_stream_id as u32)
		}
	}

	pub fn read<R: Read>(reader: &mut R, last_header: Option<Header>) -> Result<Header, ()> {
		let basic_header = reader.read_u8().unwrap();
		let fmt = (basic_header >> 6) & 0x03;
		let chunk_stream_id = Self::read_chunk_stream_id(reader, basic_header & 0x3f).unwrap();
		match fmt {
			0x00 => Self::read_header0(reader, chunk_stream_id, last_header),
			0x01 => Self::read_header1(reader, chunk_stream_id, last_header),
			0x02 => Self::read_header2(reader, chunk_stream_id, last_header),
			0x03 => Self::read_header3(reader, chunk_stream_id, last_header),
			_ => {
				Err(())
			},
		}
	}

	pub fn write<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
		let mut result = 0;
		writer.write_u8(self.chunk_stream_id as u8).unwrap();
		result += 1;
		writer.write_int::<BigEndian>(self.timestamp as i64, 3).unwrap();
		result += 3;
		writer.write_int::<BigEndian>(self.message_length as i64, 3).unwrap();
		result += 3;
		writer.write_u8(self.message_type_id).unwrap();
		result += 1;
		writer.write_u32::<BigEndian>(self.message_stream_id & 0x7fff).unwrap();
		result += 4;
		//trace!("Send of a header: {:?}", self);
		Ok(result)
	}
}

impl ::std::fmt::Debug for Header {
	fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		fmt.debug_struct("Header")
		.field("chunk_stream_id", &self.chunk_stream_id)
		.field("timestamp", &self.timestamp)
		.field("deltaTimestamp", &self.delta_timestamp)
		.field("message_length", &self.message_length)
		.field("message_type_id", &self.message_type_id)
		.field("message_stream_id", &self.message_stream_id)
		.field("type", &self.htype)
		.finish()
	}
}