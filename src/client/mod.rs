use std::io;

use bytes::{BytesMut};
use futures::{Future};
use futures::future::ok;
use message;
use std::net::SocketAddr;
use tokio_core::reactor::Core;
use tokio_io::codec::Framed;
use tokio_io::codec::{Encoder, Decoder};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::{TcpClient};
use tokio_proto::pipeline::{ClientProto};
use tokio_service::Service;

use super::{wrap_size, unwrap_size};

#[derive(Debug)]
pub struct Config {
    pub remote_addr: SocketAddr,
}


pub struct RemnantCodec;

impl Encoder for RemnantCodec {
    type Item = message::Request;
    type Error = io::Error;

    fn encode(&mut self, item: message::Request, dst: &mut BytesMut) -> Result<(), io::Error> {
        wrap_size(&item, dst)
    }
}

impl Decoder for RemnantCodec {
    type Item = message::Response;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<message::Response>, io::Error> {
        unwrap_size(src)
    }
}

#[derive(Debug)]
pub struct RemnantProto;

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for RemnantProto {
    type Request = message::Request;
    type Response = message::Response;

    type Transport = Framed<T, RemnantCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(RemnantCodec))
    }
}

pub fn main(cfg: &Config) {
    println!("Remote address: {:?}", cfg.remote_addr);

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let client = TcpClient::new(RemnantProto)
        .connect(&cfg.remote_addr, &handle)
        .and_then(|client_service| {
            client_service
                .call(message::Request::Ping)
                .and_then(|r| {
                    println!("r {:?}", r);
                    ok(r)
                })
                .map_err(|e| {
                    println!("err: {:?}", e);
                    e
                })
        });

    core.run(client).unwrap();
}
