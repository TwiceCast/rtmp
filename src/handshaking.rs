use std::io::{Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use error::Result;
use rand::{Rng, self};

#[derive(Debug)]
pub struct F0 {
  pub version: u8,
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

pub struct F1 {
  pub time: u32,
  pub zero: u32,
  pub bytes: [u8; 1528],
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

pub fn do_server_handshaking<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<()>
{
  // read c0 packet
  let c0 = F0::read(&mut reader).unwrap();
  println!("{:?}", c0);

  // read c1 packet
  let c1 = F1::read(&mut reader).unwrap();
  //println!("{:?}", c1);

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
  // read c1 packet
  let _c2 = F1::read(&mut reader).unwrap();
  Ok(())
}

pub fn do_client_handshaking<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<()>
{
  let c0 = F0 {
    version: 3
  };
  try!(c0.write(&mut writer));

  let mut rng = rand::thread_rng();
  let mut bytes: [u8; 1528] = unsafe { ::std::mem::uninitialized() };
  rng.fill_bytes(&mut bytes);
  let c1 = F1 {
    time: 0,
    zero: 0,
    bytes: bytes,
  };
  try!(c1.write(&mut writer));
  writer.flush().unwrap();
  let s0 = F0::read(&mut reader).unwrap();
  println!("{:?}", s0);
  let _s1 = F1::read(&mut reader).unwrap();
  let c2 = F1 {
    time: c1.time,
    zero: c1.time + 2000,
    bytes: c1.bytes,
  };
  try!(c2.write(&mut writer));
  let _s2 = F1::read(&mut reader).unwrap();
  Ok(())
}