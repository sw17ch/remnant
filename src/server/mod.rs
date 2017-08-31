use std::io;
use std::net::SocketAddr;
use std::str;

use bytes::{BytesMut};
use futures::{future, Future, BoxFuture};
use tokio_io::codec::Framed;
use tokio_io::codec::{Encoder, Decoder};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::TcpServer;
use tokio_proto::pipeline::ServerProto;
use tokio_service::Service;
use message;

use super::{wrap_size, unwrap_size};

#[derive(Debug)]
pub struct Config {
    pub local_addr: SocketAddr,
}

pub struct RemnantCodec;

impl Encoder for RemnantCodec {
    type Item = message::Response;
    type Error = io::Error;

    fn encode(&mut self, item: message::Response, dst: &mut BytesMut) -> Result<(), io::Error> {
        wrap_size(&item, dst)
    }
}

impl Decoder for RemnantCodec {
    type Item = message::Request;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<message::Request>, io::Error> {
        unwrap_size(src)
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
            message::Request::Empty => future::ok(message::Response::Empty).boxed(),
            message::Request::Ping => future::ok(message::Response::Ping).boxed(),
        }
    }
}

pub fn main(cfg: &Config) {
    println!("Listen address: {:?}", cfg.local_addr);

    let server = TcpServer::new(RemnantProto, cfg.local_addr);
    server.serve(|| Ok(RemnantService));
}
