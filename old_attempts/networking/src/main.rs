#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate remnant;

use docopt::Docopt;
use remnant::{server, client};
use std::net::{ToSocketAddrs};

const USAGE: &'static str = "
A distributed timeline.

Usage:
  remnant -h
  remnant --version
  remnant server [[--local-addr=<l>]]
  remnant client [[--remote-addr=<r>]]

Options:
  -h --help              Show this screen.
  --version              Show version.
  -l --local-addr=<la>   Server address to bind to [default: 0.0.0.0:9000].
  -r --remote-addr=<ra>  Remote server to connect to [default: localhost:9000].
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_version: bool,
    cmd_server: Option<ArgsServer>,
    cmd_client: Option<ArgsClient>,
}

#[derive(Debug, Deserialize)]
pub struct ArgsServer {
    flag_local_addr: String,
}

#[derive(Debug, Deserialize)]
pub struct ArgsClient {
    flag_remote_addr: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("remnant version {}", env!("CARGO_PKG_VERSION"));
    } else {
        args.cmd_client.map(|c| client::main(&client_config(&c)));
        args.cmd_server.map(|s| server::main(&server_config(&s)));
    }
}

fn server_config(args: &ArgsServer) -> server::Config {
    let sa = args.flag_local_addr
        .to_socket_addrs()
        .expect("failed to resolve local address")
        .nth(0)
        .unwrap();

    server::Config { local_addr: sa }
}

fn client_config(args: &ArgsClient) -> client::Config {
    let sa = args.flag_remote_addr
        .to_socket_addrs()
        .expect("failed to resolve remote address")
        .nth(0)
        .unwrap();


    client::Config { remote_addr: sa }
}
