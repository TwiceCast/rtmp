use std::fmt;
use std::io::{Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

use header::Header;
use usercontrolmessage::UserControlMessage;
use commandemessage::CommandeMessage;
use datamessage::DataMessage;
use audiodata::AudioData;
use videodata::VideoData;

pub static mut CHUNK_SIZE: u32 = 2048;

#[derive(Debug)]
pub enum ControleMessage {
	SetChunkSize(u32),
	Acknowledgement(u32),
	WindowsAcknowledgementSize(u32),
	SetBandwith(u32),
}

impl ControleMessage {
	pub fn read<R: Read>(header: & Header, reader: &mut R) -> Result<ControleMessage, ()> {
		match header.message_type_id {
			1 => {
				let size = reader.read_u32::<BigEndian>().unwrap();
				Ok(ControleMessage::SetChunkSize(size))
			},
			3 => {
				let sequence = reader.read_u32::<BigEndian>().unwrap();
				Ok(ControleMessage::Acknowledgement(sequence))
			},
			5 => {
				let size = reader.read_u32::<BigEndian>().unwrap();
				Ok(ControleMessage::WindowsAcknowledgementSize(size))
			},
			6 =>  {
				let bandwith = reader.read_u32::<BigEndian>().unwrap();
				let _type = reader.read_u8().unwrap();
				Ok(ControleMessage::SetBandwith(bandwith))
			},
			_ => Err(())
		}
	}

	pub fn send<W: Write>(&self, writer: &mut W) -> Result<(), ()> {
		match *self {
			ControleMessage::SetChunkSize(nb) => {
				let h = Header::new(2, 0, 4, 1, 0);
				h.write(writer).unwrap();
				writer.write_u32::<BigEndian>(nb).unwrap();
            	unsafe {
              		CHUNK_SIZE = nb;
            	}
			},
			ControleMessage::Acknowledgement(nb) => {
				let h = Header::new(2, 0, 4, 3, 0);
				h.write(writer).unwrap();
				writer.write_u32::<BigEndian>(nb).unwrap();
			},
			ControleMessage::WindowsAcknowledgementSize(nb) => {
				let h = Header::new(2, 0, 4, 5, 0);
				h.write(writer).unwrap();
				writer.write_u32::<BigEndian>(nb).unwrap();
			},
			ControleMessage::SetBandwith(nb) => {
				let h = Header::new(2, 0, 5, 6, 0);
				h.write(writer).unwrap();
				writer.write_u32::<BigEndian>(nb).unwrap();
				writer.write_u8(1).unwrap();
			},
		}
		Ok(())
	}

	pub fn write<W: Write>(&self, writer: &mut W) { 
		match *self {
			ControleMessage::SetChunkSize(nb) => writer.write_u32::<BigEndian>(nb).unwrap(),
			ControleMessage::Acknowledgement(nb) => writer.write_u32::<BigEndian>(nb).unwrap(),
			ControleMessage::WindowsAcknowledgementSize(nb) => writer.write_u32::<BigEndian>(nb).unwrap(),
			ControleMessage::SetBandwith(nb) => {writer.write_u32::<BigEndian>(nb).unwrap(); writer.write_u8(1).unwrap()},
		}
	}
}

impl fmt::Display for ControleMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
        	ControleMessage::SetChunkSize(size) => write!(f, "Set chunk size to {}", size),
        	ControleMessage::Acknowledgement(seq) => write!(f, "Acknowledgement of seq  {}", seq),
        	ControleMessage::WindowsAcknowledgementSize(seq) => write!(f, "Acknowledgement Window size {}", seq),
			ControleMessage::SetBandwith(size) => write!(f, "Set bandwith to {}", size),
        }
    }
}

pub enum Message {
	ControleMessage(ControleMessage),
	CommandeMessage(CommandeMessage),
	UserControlMessage(UserControlMessage),
	DataMessage(DataMessage),
	AudioData(AudioData),
	VideoData(VideoData),
	Unknown(Vec<u8>),
}

impl Message {
	pub fn read<R: Read>(header: &mut Header, reader: &mut R) -> Result<Message, ()> {
		match header.message_type_id {
			1 | 3 | 5 | 6 => Ok(Message::ControleMessage(ControleMessage::read(header, reader).unwrap())),
			4 => Ok(Message::UserControlMessage(UserControlMessage::read(reader).unwrap())),
			8 => Ok(Message::AudioData(AudioData::read(header, reader).unwrap())),
			9 => Ok(Message::VideoData(VideoData::read(header, reader).unwrap())),
			18 => Ok(Message::DataMessage(DataMessage::read(header, reader).unwrap())),
			20 => {
				Ok(Message::CommandeMessage(CommandeMessage::read(header, reader).unwrap()))
			},
			_ =>{				
				let mut slice = vec![];
				slice.resize(header.message_length as usize, 0);
    			reader.read(&mut slice).unwrap();
    			Ok(Message::Unknown(slice))
			},
		}
	}

	pub fn write<W: Write>(self, writer: &mut W) {
		match self {
			Message::ControleMessage(ref m) => m.write(writer),
			Message::CommandeMessage(ref m) => m.write(writer),
			Message::UserControlMessage(ref m) => m.write(writer),
			Message::DataMessage(ref m) => m.write(writer),
			Message::AudioData(ref m) => m.write(writer),
			Message::VideoData(ref m) => m.write(writer),
			Message::Unknown(ref data) => { let _  = writer.write(&data).unwrap(); },
		}
	}
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
        	Message::ControleMessage(ref mess) => write!(f, "{}", mess),
        	Message::CommandeMessage(ref mess) => write!(f, "{}", mess),
        	Message::UserControlMessage(ref mess) => write!(f, "{}", mess),
        	Message::DataMessage(ref mess) => write!(f, "{:?}", mess),
  			Message::AudioData(ref mess) => write!(f, "{:?}", mess),
  			Message::VideoData(ref mess) => write!(f, "{:?}", mess),
  			Message::Unknown(_) => write!(f, "unknown"),
        }
    }
}