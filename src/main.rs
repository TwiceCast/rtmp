extern crate byteorder;
extern crate rand;
extern crate time;

use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use rand::Rng;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

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
//      return Err("Fuck zeros".into());
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

fn handle_client(stream: TcpStream) -> Result<()> {
//  let start = time::SteadyTime::now();

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
  try!(s0.write(&mut writer));

  let mut rng = rand::thread_rng();
  let mut bytes: [u8; 1528] = unsafe { ::std::mem::uninitialized() };
  rng.fill_bytes(&mut bytes);
  let s1 = F1 {
    time: 0,
    zero: 0,
    bytes: bytes,
  };
  try!(s1.write(&mut writer));

  let s2 = F1 {
    time: c1.time,
    zero: c1.time + 2000,
    bytes: c1.bytes,
  };
  try!(s2.write(&mut writer));
  try!(writer.flush());
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
        break;
      }
    }
  }
  Ok(())
}

fn main() {
  let listener = TcpListener::bind("0.0.0.0:80").unwrap();
  let mut handle_thread = Vec::new();
//  let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

  for stream in listener.incoming() {
    match stream {
      Ok(stream) => {
        let builder = thread::Builder::new().name("handle_client".into());
//        let thread_tx = tx.clone();

        match builder.spawn(move || {
          handle_client(stream)
        }) {
          Ok (thread) => {
            handle_thread.push(thread);
          }
          Err(e) => {
            println!("{:?}", e);
          }
        }
      }
      Err(e) => {
        println!("{:?}", e);
      }
    }
  }

  for thread in handle_thread {
    match thread.join() {
      Ok(_) => {
        return;
      }
      Err(_) => {
        return;
      }
    }
  }
}
