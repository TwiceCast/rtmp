extern crate byteorder;
extern crate rand;
extern crate time;
extern crate amf;

use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::collections::BTreeMap;
use std::fs::File;
use amf::Value;

use std::thread;

use header::Header;
use message::{Message, ControleMessage};
use usercontrolmessage::UserControlMessage;
use commandemessage::{CommandeMessage, NetConnection};
use error::{Result};
use handshaking::{do_server_handshaking, do_client_handshaking};

mod header;
mod message;
mod usercontrolmessage;
mod commandemessage;
mod error;
mod handshaking;
mod datamessage;
mod audiodata;
mod videodata;

fn on_receive<W: Write>(writer: &mut W, m: & Message) -> Result<()> {
  match *m {
    Message::CommandeMessage(ref m) => {
      match *m {
        CommandeMessage::NetConnection(ref mess) => {
          if mess.command_name == "connect" {
            let rep = ControleMessage::WindowsAcknowledgementSize(50000000);
            rep.send(writer).unwrap();
            let rep = ControleMessage::SetBandwith(50000000);
            rep.send(writer).unwrap();
            let rep = ControleMessage::SetChunkSize(4096);
            rep.send(writer).unwrap();
            //let rep = UserControlMessage::StreamBegin(42);
            //rep.send(writer).unwrap();
            let mut map = BTreeMap::new();
            map.insert("capabilities".to_string(), Value::Number(32.));
            map.insert("fmsVersion".to_string(), Value::String("FMS/3,0,1,123".to_string()));
            let connect = NetConnection{ command_name: "_result".to_string(), transaction_id: mess.transaction_id, properties: Value::Object(map), opt: None };
            connect.send(writer).unwrap();
            writer.flush().unwrap();            
          }
          else if mess.command_name == "createStream" {
            let connect = NetConnection{ command_name: "_result".to_string(), transaction_id: mess.transaction_id, properties: Value::Null, opt: Some(Value::Number(42.)) };
            connect.send(writer).unwrap();
            writer.flush().unwrap();       
          }
          Ok(())
        },
        CommandeMessage::NetStreamCommand(ref mess) =>{
          if mess.command_name == "publish" {
            let mut map = BTreeMap::new();
            map.insert("level".to_string(), Value::String("status".to_string()));
            map.insert("code".to_string(), Value::String("NetStream.Publish.Start".to_string()));
            map.insert("description".to_string(), Value::String("publish of the stream".to_string()));
            let connect = NetConnection{ command_name: "onStatus".to_string(), transaction_id: 0., properties: Value::Null, opt: Some(Value::Object(map)) };
            connect.send(writer).unwrap();
            writer.flush().unwrap();   
          }
          else if mess.command_name == "deleteStream" {
            let mut map = BTreeMap::new();
            map.insert("level".to_string(), Value::String("status".to_string()));
            map.insert("code".to_string(), Value::String("NetStream.Unpublish.Success".to_string()));
            map.insert("description".to_string(), Value::String("Stop publishing".to_string()));
            let connect = NetConnection{ command_name: "onStatus".to_string(), transaction_id: 0., properties: Value::Null, opt: Some(Value::Object(map)) };
            connect.send(writer).unwrap();
            writer.flush().unwrap();
          }
          Ok(())
        }
      }
    },
    Message::ControleMessage(ref m) => {
      match *m {
        ControleMessage::SetChunkSize(nb) => {
          unsafe {
            message::CHUNK_SIZE = nb;
          }
          Ok(())
        },
        _ => Ok(())
      }
    }
    _ => Ok(())
  }
}

fn handle_client(stream: TcpStream) -> Result<()> {
//  let start = time::SteadyTime::now();
  let mut file = File::create("log.txt").unwrap();
  let mut last_header = None;
  println!("{:?}, {:?}", stream.peer_addr(), stream.local_addr());
  let mut reader = BufReader::new(&stream);
  let mut writer = BufWriter::new(&stream);
  do_server_handshaking(&mut reader, &mut writer).unwrap();
  println!("Handshaking Okay");
  loop {
    let mut h = Header::read(&mut reader, last_header).unwrap();
    let m = Message::read(&mut h, &mut reader).unwrap();
    match m {
      Message::AudioData(_) => println!("{:?}", h),
      Message::VideoData(_) => println!("{:?}", h),
      _ => {
        println!("{:?}", h);
        println!("{}", m);
        println!(""); 
      }
    }
    on_receive(&mut writer, &m).unwrap();
    match m {
      Message::Unknown(_) => (),
      _ => last_header = Some(h),
    }
    {
      h.write(&mut file).unwrap();
      m.write(&mut file);
    }
    writer.flush().unwrap();
  }
}

fn handle_server(stream: TcpStream) {
  let mut last_header = None;
  println!("{:?}, {:?}", stream.peer_addr(), stream.local_addr());
  let mut reader = BufReader::new(&stream);
  let mut writer = BufWriter::new(&stream);
  do_client_handshaking(&mut reader, &mut writer).unwrap();
  let m = ControleMessage::SetChunkSize(4096);
  m.send(&mut writer).unwrap();
  let mut map = BTreeMap::new();
  map.insert("app".to_string(), Value::String("live".to_string()));
  map.insert("flashVer".to_string(), Value::String("FMLE/3.0 (compatible; FMSc/1.0)".to_string()));
  map.insert("swfUrl".to_string(), Value::String("rtmp://37.187.99.70:1935/live".to_string()));
  map.insert("tcUrl".to_string(), Value::String("rtmp://37.187.99.70:1935/live".to_string()));
  map.insert("type".to_string(), Value::String("nonprivate".to_string()));
  let m2 = NetConnection{ command_name: "connect".to_string(), transaction_id: 1., properties: Value::Object(map), opt: None };
  m2.send(&mut writer).unwrap();
  writer.flush().unwrap();
  loop {
    let mut h = Header::read(&mut reader, last_header).unwrap();
    println!("{:?}", h);
    let m = Message::read(&mut h, &mut reader).unwrap();
    println!("{}", m);
    match m {
      Message::UserControlMessage(m) => {
        match m {
          UserControlMessage::PingRequest(t) => {
            let rep = UserControlMessage::PingResponse(t);
            rep.send(&mut writer).unwrap();
            writer.flush().unwrap();
          },
          _ => (),          
        }
      }
      _ => (),
    };
    last_header = Some(h);
  }
}

fn main_client() {
  let stream = TcpStream::connect("37.187.99.70:1935").unwrap();
  handle_server(stream)
}

fn main_server() {
//  println!("server");
  let listener = TcpListener::bind("0.0.0.0:42").unwrap();
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

fn main() {
  main_server();
  main_client()
}