use std::fmt;
use std::io::{Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

use header::Header;
use usercontrolmessage::UserControlMessage;
use commandemessage::CommandeMessage;
use datamessage::DataMessage;
use audiodata::AudioData;
use videodata::VideoData;
use slicereader::SliceReader;
use messageinterface::MessageInterface;

pub static mut CHUNK_SIZE: u32 = 128;
static mut LAST_HEADER: Option<Header> = None;

#[derive(Debug)]
pub struct SetChunkSize {
	pub size: u32,
}

#[derive(Debug)]
pub struct Acknowledgement {
	pub sequence: u32,
}

#[derive(Debug)]
pub struct WindowsAcknowledgementSize {
	pub size: u32,
}

#[derive(Debug)]
pub struct SetBandwith {
	pub bandwith: u32,
	pub limit_type: u8,
}

#[derive(Debug)]
pub enum ControleMessage {
	SetChunkSize(SetChunkSize),
	Acknowledgement(Acknowledgement),
	WindowsAcknowledgementSize(WindowsAcknowledgementSize),
	SetBandwith(SetBandwith),
}

impl MessageInterface for SetChunkSize {
	type Message = SetChunkSize;

	fn read<R: Read>(_header: Header, reader: &mut R) -> Result<Self::Message, ()> {
		let size = reader.read_u32::<BigEndian>().unwrap();
		let ret = SetChunkSize{size: size};
		debug!("Reception of {}", ret);
		Ok(ret)
	}

	fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
		let mut result = 0;
		writer.write_u32::<BigEndian>(self.size).unwrap();
		result += 4;
		unsafe {
			CHUNK_SIZE = self.size;
		}
		debug!("Send of {}", self);
		Ok(result)
	}

	fn fill_header(&self, header: &mut Header) {
		header.copy(Header::new(2, 0, 4, 1, 0));
	}
}

impl fmt::Display for SetChunkSize {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Set chunk size to {}", self.size)
	}
}

impl MessageInterface for Acknowledgement {
	type Message = Acknowledgement;

	fn read<R: Read>(_header: Header, reader: &mut R) -> Result<Self::Message, ()> {
		let sequence = reader.read_u32::<BigEndian>().unwrap();
		let ret = Acknowledgement{sequence: sequence};
		debug!("Reception of {}", ret);
		Ok(ret)
	}

	fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
		let mut result = 0;
		writer.write_u32::<BigEndian>(self.sequence).unwrap();
		result += 4;
		debug!("Send of {}", self);
		Ok(result)
	}

	fn fill_header(&self, header: &mut Header) {
		header.copy(Header::new(2, 0, 4, 3, 0));
	}
}

impl fmt::Display for Acknowledgement {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Acknowledgement of seq  {}", self.sequence)
	}
}

impl MessageInterface for WindowsAcknowledgementSize {
	type Message = WindowsAcknowledgementSize;

	fn read<R: Read>(_header: Header, reader: &mut R) -> Result<Self::Message, ()> {
		let size = reader.read_u32::<BigEndian>().unwrap();
		let ret = WindowsAcknowledgementSize{size: size};
		debug!("Reception of {}", ret);
		Ok(ret)
	}

	fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
		let mut result = 0;
		writer.write_u32::<BigEndian>(self.size).unwrap();
		result += 4;
		debug!("Send of {}", self);
		Ok(result)
	}

	fn fill_header(&self, header: &mut Header) {
		header.copy(Header::new(2, 0, 4, 5, 0));
	}
}

impl fmt::Display for WindowsAcknowledgementSize {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Acknowledgement Window size {}", self.size)
	}
}

impl MessageInterface for SetBandwith {
	type Message = SetBandwith;

	fn read<R: Read>(_header: Header, reader: &mut R) -> Result<Self::Message, ()> {
		let bandwith = reader.read_u32::<BigEndian>().unwrap();
		let limit_type = reader.read_u8().unwrap();
		let ret = SetBandwith{bandwith: bandwith, limit_type: limit_type};
		debug!("Reception of {}", ret);
		Ok(ret)
	}

	fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
		let mut result = 0;
		writer.write_u32::<BigEndian>(self.bandwith).unwrap();
		result += 4;
		writer.write_u8(self.limit_type).unwrap();
		result += 1;
		debug!("Send of {}", self);
		Ok(result)
	}

	fn fill_header(&self, header: &mut Header) {
		header.copy(Header::new(2, 0, 5, 6, 0));
	}
}

impl fmt::Display for SetBandwith {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Set bandwith to {}", self.bandwith)
	}
}

impl MessageInterface for ControleMessage {
	type Message = ControleMessage;
	fn read<R: Read>(header: Header, reader: &mut R) -> Result<Self::Message, ()> {
		match header.message_type_id {
			1 => {
				Ok(ControleMessage::SetChunkSize(SetChunkSize::read(header, reader).unwrap()))
			},
			3 => {
				Ok(ControleMessage::Acknowledgement(Acknowledgement::read(header, reader).unwrap()))
			},
			5 => {
				Ok(ControleMessage::WindowsAcknowledgementSize(WindowsAcknowledgementSize::read(header, reader).unwrap()))
			},
			6 =>  {
				Ok(ControleMessage::SetBandwith(SetBandwith::read(header, reader).unwrap()))
			},
			_ => Err(())
		}
	}

	fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()> {
		match *self {
			ControleMessage::SetChunkSize(ref s) => {
				s.send(writer)
			},
			ControleMessage::Acknowledgement(ref s) => {
				s.send(writer)
			},
			ControleMessage::WindowsAcknowledgementSize(ref s) => {
				s.send(writer)
			},
			ControleMessage::SetBandwith(ref s) => {
				s.send(writer)
			},
		}
	}

	fn fill_header(&self, header: &mut Header) {
		match *self {
			ControleMessage::SetChunkSize(ref s) => {
				s.fill_header(header)
			},
			ControleMessage::Acknowledgement(ref s) => {
				s.fill_header(header)
			},
			ControleMessage::WindowsAcknowledgementSize(ref s) => {
				s.fill_header(header)
			},
			ControleMessage::SetBandwith(ref s) => {
				s.fill_header(header)
			},
		}
	}

/*	pub fn write<W: Write>(&self, writer: &mut W) -> Result<usize, ()> { 
		match *self {
			ControleMessage::SetChunkSize(nb) => {writer.write_u32::<BigEndian>(nb).unwrap(); return Ok(4)},
			ControleMessage::Acknowledgement(nb) => {writer.write_u32::<BigEndian>(nb).unwrap(); return Ok(4)},
			ControleMessage::WindowsAcknowledgementSize(nb) => {writer.write_u32::<BigEndian>(nb).unwrap(); return Ok(4)},
			ControleMessage::SetBandwith(nb) => {writer.write_u32::<BigEndian>(nb).unwrap(); writer.write_u8(1).unwrap(); return Ok(5)},
		}
	}*/
}

impl fmt::Display for ControleMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ControleMessage::SetChunkSize(ref s) => write!(f, "{}", s),
			ControleMessage::Acknowledgement(ref s) => write!(f, "{}", s),
			ControleMessage::WindowsAcknowledgementSize(ref s) => write!(f, "{}", s),
			ControleMessage::SetBandwith(ref s) => write!(f, "{}", s),
		}
	}
}

pub enum Message {
	ControleMessage(Header, ControleMessage),
	CommandeMessage(Header, CommandeMessage),
	UserControlMessage(Header, UserControlMessage),
	DataMessage(Header, DataMessage),
	AudioData(Header, AudioData),
	VideoData(Header, VideoData),
	Unknown(Header, Vec<u8>),
}

impl fmt::Display for Message {

	/*
	** @Todo display the header with the message
	*/
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Message::ControleMessage(ref _h, ref mess) => write!(f, "{}", mess),
			Message::CommandeMessage(ref _h, ref mess) => write!(f, "{}", mess),
			Message::UserControlMessage(ref _h, ref mess) => write!(f, "{}", mess),
			Message::DataMessage(ref _h, ref mess) => write!(f, "{:?}", mess),
			Message::AudioData(ref _h, ref mess) => write!(f, "{:?}", mess),
			Message::VideoData(ref _h, ref mess) => write!(f, "{:?}", mess),
			Message::Unknown(ref _h, _) => write!(f, "unknown"),
		}
	}
}

impl Message {
	pub fn read<R: Read>(reader: &mut R) -> Result<Message, ()>
	{
		let header;
		unsafe {
			//debug!("Reading of a header. Last Header {:?}", LAST_HEADER);
			header = Header::read(reader, LAST_HEADER).unwrap();
			LAST_HEADER = Some(header);
			//debug!("Header read: {:?}", header);
		}
		let mut slice = vec![];
		let mut size = header.message_length;
		let mut tmp_header = header;
		unsafe {
			while size > 0 {
				let mut buf = vec![];
				buf.resize(if size < CHUNK_SIZE { size } else { CHUNK_SIZE } as usize, 0);
				size -= reader.read(&mut buf).unwrap() as u32;
				slice.extend_from_slice(&buf);
				if size > 0 { tmp_header = Header::read(reader, Some(tmp_header)).unwrap() };
				//debug!("tmp Header read: {:?}", tmp_header);
			}
		}
		trace!("data read {:?}", slice);
		let mut slice_reader = SliceReader::new(&slice);
		match header.message_type_id {
			1 | 3 | 5 | 6 => Ok(Message::ControleMessage(header, ControleMessage::read(header, &mut slice_reader).unwrap())),
			4 => Ok(Message::UserControlMessage(header, UserControlMessage::read(header, &mut slice_reader).unwrap())),
			8 => Ok(Message::AudioData(header, AudioData::read(header, &mut slice_reader).unwrap())),
			9 => Ok(Message::VideoData(header, VideoData::read(header, &mut slice_reader).unwrap())),
			18 => Ok(Message::DataMessage(header, DataMessage::read(header, &mut slice_reader).unwrap())),
			20 => {
				Ok(Message::CommandeMessage(header, CommandeMessage::read(header, &mut slice_reader).unwrap()))
			},
			_ =>{
				let mut slice = vec![];
				slice.resize(header.message_length as usize, 0);
				reader.read(&mut slice).unwrap();
				warn!("Reception of an Unknown message");
				Ok(Message::Unknown(header, slice))
			},
		}
	}

	pub fn send<W: Write>(&mut self, writer: &mut W) -> Result<usize, ()>
	{
		let mut result = 0;
		match *self {
			Message::ControleMessage(ref mut h, ref m) => { m.fill_header(h); result += h.write(writer).unwrap(); result += m.send(writer).unwrap()},
			Message::CommandeMessage(ref mut h, ref m) => { m.fill_header(h); result += h.write(writer).unwrap(); result += m.send(writer).unwrap()},
			Message::UserControlMessage(ref mut h, ref m) => { m.fill_header(h); result += h.write(writer).unwrap(); result += m.send(writer).unwrap()},
			Message::DataMessage(ref mut h, ref m) => { m.fill_header(h); result += h.write(writer).unwrap(); result += m.send(writer).unwrap()},
			Message::AudioData(ref mut h, ref m) => { m.fill_header(h); result += h.write(writer).unwrap(); result += m.send(writer).unwrap()},
			Message::VideoData(ref mut h, ref m) => { m.fill_header(h); result += h.write(writer).unwrap(); result += m.send(writer).unwrap()},
			Message::Unknown(_, _) => return Err(()),
		}
		Ok(result)
	}
}