extern crate byteorder;
extern crate rand;
extern crate time;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::str;
use rand::Rng;

#[derive(Debug)]
enum Error {
  Io(::std::io::Error),
  Other(String),
}

impl From<::std::io::Error> for Error {
  fn from(err: ::std::io::Error) -> Error {
    Error::Io(err)
  }
}

impl From<String> for Error {
  fn from(err: String) -> Error {
    Error::Other(err)
  }
}

impl<'a> From<&'a str> for Error {
  fn from(err: &'a str) -> Error {
    Error::Other(err.to_owned())
    // Alias: Error::from(err.to_owned())
    // Alias: err.to_owned().into()
  }
}

type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
struct F0 {
  version: u8,
}

impl F0 {
  pub fn read<R: Read>(reader: &mut R) -> Result<F0> {
    let version = try!(reader.read_u8());

    Ok(F0 {
      version: version,
    })
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
    try!(writer.write_u8(self.version));

    Ok(())
  }
}

struct F1 {
  time: u32,
  zero: u32,
  bytes: [u8; 1528],
}

impl F1 {
  pub fn read<R: Read>(reader: &mut R) -> Result<F1> {
    let time = try!(reader.read_u32::<BigEndian>());
    let zero = try!(reader.read_u32::<BigEndian>());
    if zero != 0 {
      return Err("Fuck zeros".into());
    }
    let mut bytes: [u8; 1528] = unsafe { ::std::mem::uninitialized() };
    try!(reader.read_exact(&mut bytes));

    Ok(F1 {
      time: time,
      zero: zero,
      bytes: bytes,
    })
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
    try!(writer.write_u32::<BigEndian>(self.time));
    try!(writer.write_u32::<BigEndian>(self.zero));
    try!(writer.write(&self.bytes));

    Ok(())
  }
}

impl ::std::fmt::Debug for F1 {
  fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    let bytes = &self.bytes as &[u8];
    fmt.debug_struct("F1")
       .field("time", &self.time)
       .field("zero", &self.zero)
       .field("bytes", &bytes)
       .finish()
  }
}

fn handle_client(stream: TcpStream) {
  let start = SteadyTime::now();

  println!("{:?}, {:?}", stream.peer_addr(), stream.local_addr());
  let mut reader = BufReader::new(&stream);
  let mut writer = BufWriter::new(&stream);

  // read c0 packet
  let c0 = F0::read(&mut reader).unwrap();
  println!("{:?}", c0);

  // read c1 packet
  let c1 = F1::read(&mut reader).unwrap();
  println!("{:?}", c1);

  let s0 = F0 {
    version: 3,
  };
  s0.write(&mut writer);

  let mut rng = rand::thread_rng();
  let mut bytes: [u8; 1528] = unsafe { ::std::mem::uninitialized() };
  rng.fill_bytes(&mut bytes);
  let s1 = F1 {
    time: timestamp(),
    zero: 0,
    bytes: bytes,
  };
  s1.write(&mut writer);
  loop {
    match reader.read(&mut bytes) {
      Ok(size) => {
        if size == 0 {
          break;
        }
        println!("loop = {:?}", &bytes[0..size]);
      }
      Err(e) => {
        println!("{:?}", e);
      }
    }
  }
}

fn main() {
  let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

  for stream in listener.incoming() {
    match stream {
      Ok(stream) => {
        thread::spawn(move|| {
          handle_client(stream)
        });
      }
      Err(e) => {
        println!("{:?}", e);
      }
    }
  }
}
