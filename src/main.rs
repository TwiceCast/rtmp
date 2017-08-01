extern crate byteorder;
extern crate rand;
extern crate time;
extern crate amf;
#[macro_use]
extern crate log;
extern crate simple_logger;

use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::collections::BTreeMap;
use amf::Value;

use std::thread;

use message::{Message, ControleMessage, SetChunkSize, WindowsAcknowledgementSize, SetBandwith};
use usercontrolmessage::UserControlMessage;
use commandemessage::{CommandeMessage, NetConnection};
use error::{Result};
use handshaking::{do_server_handshaking, do_client_handshaking};
use messageinterface::MessageInterface;
use header::Header;

mod header;
mod message;
mod usercontrolmessage;
mod commandemessage;
mod error;
mod handshaking;
mod datamessage;
mod audiodata;
mod videodata;
mod slicereader;
mod messageinterface;

fn on_receive<W: Write>(writer: &mut W, m: & Message) -> Result<()> {
  match *m {
    Message::CommandeMessage(ref _h, ref m) => {
      match *m {
        CommandeMessage::NetConnection(ref mess) => {
          if mess.command_name == "connect" {
            let header = Header::new(0, 0, 0, 0, 0);
            let mut rep = Message::ControleMessage(header, ControleMessage::WindowsAcknowledgementSize(WindowsAcknowledgementSize{size: 50000000}));
            rep.send(writer).unwrap();
/*            let mut rep = Message::ControleMessage(header, ControleMessage::SetBandwith(SetBandwith{bandwith: 50000000, limit_type: 1}));
            rep.send(writer).unwrap();*/
            let mut rep = Message::ControleMessage(header, ControleMessage::SetChunkSize(SetChunkSize{size: 4096}));
            rep.send(writer).unwrap();
            //let rep = UserControlMessage::StreamBegin(42);
            //rep.send(writer).unwrap();
            let mut map = BTreeMap::new();
            map.insert("capabilities".to_string(), Value::Number(31.));
            map.insert("fmsVersion".to_string(), Value::String("FMS/3,0,1,123".to_string()));
            let mut opt = BTreeMap::new();
            opt.insert("level".to_string(), Value::String("status".to_string()));
            opt.insert("code".to_string(), Value::String("NetConnection.Connection.Success".to_string()));
            opt.insert("description".to_string(), Value::String("connection succeeded".to_string()));
            opt.insert("objectEncoding".to_string(), Value::Number(0.));
            let mut connect = Message::CommandeMessage(header, CommandeMessage::NetConnection(NetConnection::new("_result".to_string(), mess.transaction_id, Value::Object(map), Some(Value::Object(opt)))));
            connect.send(writer).unwrap();
            writer.flush().unwrap();            
          }
          else if mess.command_name == "createStream" {
            let header = Header::new(0, 0, 0, 0, 0);
            let mut connect = Message::CommandeMessage(header, CommandeMessage::NetConnection(NetConnection::new("_result".to_string(), mess.transaction_id, Value::Null, Some(Value::Number(42.)))));
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
            let header = Header::new(0, 0, 0, 0, 0);
            let mut connect = Message::CommandeMessage(header, CommandeMessage::NetConnection(NetConnection::new( "onStatus".to_string(), 0., Value::Null, Some(Value::Object(map)))));
            connect.send(writer).unwrap();
            writer.flush().unwrap();   
          }
          else if mess.command_name == "deleteStream" {
            let header = Header::new(0, 0, 0, 0, 0);
            let mut map = BTreeMap::new();
            map.insert("level".to_string(), Value::String("status".to_string()));
            map.insert("code".to_string(), Value::String("NetStream.Unpublish.Success".to_string()));
            map.insert("description".to_string(), Value::String("Stop publishing".to_string()));
            let mut connect = Message::CommandeMessage(header, CommandeMessage::NetConnection(NetConnection::new("onStatus".to_string(), 0., Value::Null, Some(Value::Object(map)))));
            connect.send(writer).unwrap();
            writer.flush().unwrap();
          }
          Ok(())
        }
      }
    },
    Message::ControleMessage(ref _h, ref m) => {
      match *m {
        ControleMessage::SetChunkSize(ref s) => {
          unsafe {
            info!("changement of the chunk size to {}", s.size);
            message::CHUNK_SIZE = s.size;
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
  let mut reader = BufReader::new(&stream);
  let mut writer = BufWriter::new(&stream);
  do_server_handshaking(&mut reader, &mut writer).unwrap();
  debug!("Handshaking done");
  loop {
    let m = Message::read(&mut reader).unwrap();
    println!("{}", m);
    on_receive(&mut writer, &m).unwrap();
    writer.flush().unwrap();
  }
}

fn handle_server(stream: TcpStream) {
  let mut reader = BufReader::new(&stream);
  let mut writer = BufWriter::new(&stream);
  do_client_handshaking(&mut reader, &mut writer).unwrap();
  debug!("Handshaking done");
  let m = ControleMessage::SetChunkSize(SetChunkSize{size: 4096});
  m.send(&mut writer).unwrap();
  let mut map = BTreeMap::new();
  map.insert("app".to_string(), Value::String("live".to_string()));
  map.insert("flashVer".to_string(), Value::String("FMLE/3.0 (compatible; FMSc/1.0)".to_string()));
  map.insert("swfUrl".to_string(), Value::String("rtmp://37.187.99.70:1935/live".to_string()));
  map.insert("tcUrl".to_string(), Value::String("rtmp://37.187.99.70:1935/live".to_string()));
  map.insert("type".to_string(), Value::String("nonprivate".to_string()));
  let m2 = NetConnection::new("connect".to_string(), 1., Value::Object(map), None);
  m2.send(&mut writer).unwrap();
  writer.flush().unwrap();
  loop {
    let m = Message::read(&mut reader).unwrap();
    println!("{}", m);
    match m {
      Message::UserControlMessage(_h, m) => {
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
  }
}

fn main_client() {
  let stream = TcpStream::connect("37.187.99.70:1935").unwrap();
  handle_server(stream)
}

fn main_server() {
  info!("server start");
  let listener = TcpListener::bind("0.0.0.0:42").unwrap();
  let mut handle_thread = Vec::new();
//  let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

  for stream in listener.incoming() {
    match stream {
      Ok(stream) => {
        info!("New client connected");
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
  simple_logger::init_with_level(log::LogLevel::Trace).unwrap();
  main_server();
  main_client()
}