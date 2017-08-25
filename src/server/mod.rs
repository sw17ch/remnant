use std::io;
use std::io::Cursor;
use std::str;
use std::net::SocketAddr;
use std::mem::size_of;

use bincode::{serialize, deserialize, Infinite};
use bytes::{BufMut, BytesMut, BigEndian};
use byteorder::ReadBytesExt;
use byteorder::BigEndian as BOBigEndian;
use futures::{future, Future, BoxFuture};
use tokio_io::codec::Framed;
use tokio_io::codec::{Encoder, Decoder};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::TcpServer;
use tokio_proto::pipeline::ServerProto;
use tokio_service::Service;
use message;

#[derive(Debug)]
pub struct Config {
    pub local_addr: SocketAddr,
}

#[derive(Default)]
pub struct RemnantCodec;

impl Encoder for RemnantCodec {
    type Item = message::Response;
    type Error = io::Error;

    fn encode(&mut self, item: message::Response, dst: &mut BytesMut) -> Result<(), io::Error> {
        println!("ENC: {:?}", item);

        let enc = serialize(&item, Infinite).unwrap();
        dst.put_u64::<BigEndian>(enc.len() as u64);
        dst.extend(enc);
        Ok(())
    }
}

impl Decoder for RemnantCodec {
    type Item = message::Request;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<message::Request>, io::Error> {
        let header_size = size_of::<u64>();
        let total_len = src.len();

        if total_len < header_size {
            Ok(None)
        } else {
            let len = Cursor::new(&src).read_u64::<BOBigEndian>().unwrap() as usize;

            if len == 0 {
                /// A length of 0 is invalid.
                println!("ERROR zero length");
                Err(io::Error::new(io::ErrorKind::Other, "zero length"))
            } else if len + header_size > total_len {
                /// The buffer doesn't have enough data yet.
                println!("PENDING");
                Ok(None)
            } else {
                let msg = src.split_to(header_size + len);
                let d = &msg[header_size..(header_size + len)];

                match deserialize(d) {
                    Ok(resp) => {
                        println!("DEC: {:?}", resp);
                        Ok(Some(resp))
                    }
                    Err(e) => {
                        println!("ERROR bad decode [{:?}]", e);
                        Err(io::Error::new(io::ErrorKind::Other, "bad decode"))
                    }
                }
            }
        }
    }
}

pub struct RemnantProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for RemnantProto {
    type Request = message::Request;
    type Response = message::Response;

    type Transport = Framed<T, RemnantCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(RemnantCodec))
    }
}

pub struct RemnantService;

impl Service for RemnantService {
    type Request = message::Request;
    type Response = message::Response;

    type Error = io::Error;

    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            message::Request::Ping => future::ok(message::Response::Ping).boxed(),
        }
    }
}

pub fn main(cfg: &Config) {
    println!("Listen address: {:?}", cfg.local_addr);

    let server = TcpServer::new(RemnantProto, cfg.local_addr);
    server.serve(|| Ok(RemnantService));
}
