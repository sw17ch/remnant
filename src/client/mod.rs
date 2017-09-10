use bytes::BytesMut;
use futures::future::{ok};
use futures::sync::mpsc;
use futures::{Future, Stream, Sink};
use message;
use std::io;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::thread;
use std;
use tokio_core::reactor::Core;
use tokio_io::codec::Framed;
use tokio_io::codec::{Encoder, Decoder};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::{ClientProto};
use tokio_proto::TcpClient;
use tokio_service::Service;

#[derive(Debug)]
pub struct Config {
    pub remote_addr: SocketAddr,
}


pub struct RemnantCodec;

impl Encoder for RemnantCodec {
    type Item = message::Request;
    type Error = io::Error;

    fn encode(&mut self, item: message::Request, dst: &mut BytesMut) -> Result<(), io::Error> {
        super::wrap_size(&item, dst)
    }
}

impl Decoder for RemnantCodec {
    type Item = message::Response;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<message::Response>, io::Error> {
        super::unwrap_size(src)
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

fn ui_thread(mut ui_tx: mpsc::Sender<String>) -> () {
    let stdin = std::io::stdin();

    loop {
        let mut inbuf = String::new();
        if 0 == stdin.read_line(&mut inbuf).unwrap_or(0) {
            // stdin closed or we read 0 bytes.
            break;
        } else if "done" == inbuf.trim() {
            break;
        }

        ui_tx = match ui_tx.send(inbuf).wait() {
            Ok(x) => {
                println!("sent");
                x
            }
            Err(e) => {
                println!("err: {}", e);
                panic!()
            }
        }
    }

    println!("UI Thread exiting");
}

pub fn main(cfg: &Config) {
    println!("Remote address: {:?}", cfg.remote_addr);

    let (ui_tx, ui_rx) = mpsc::channel(0);
    let ui_future = ui_rx.into_future();

    let ui_thread_join = thread::spawn(|| ui_thread(ui_tx));

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // let svc:

    let client = TcpClient::new(RemnantProto)
        .connect(&cfg.remote_addr, &handle)
        .and_then(|client_service| {
            // The `client_service` has a type of
            // `ClientService<TcpStream,RemnantProto>` at this
            // point. The `Service` trait is implemented for
            // `ClientService<T,P>`.
            //
            // When the service becomes available, we wait for input
            // from the command line UI. Once that command is
            // available, we interact with the client_service based on
            // that input.
            ui_future
                .map_err(|(e, r)| {
                    Error::new(ErrorKind::Other, format!("oh no ({:?},{:?})", e, r))
                })
                .and_then(move |(item, _)| {
                    println!("item {:?}!", item);
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
                })
        });

    let result = core.run(client).unwrap();
    println!("Main thread done. {:?}", result);

    ui_thread_join.join().unwrap();
}
