use std::net::{SocketAddr};

#[derive(Debug)]
pub struct Config {
    pub remote_addr: SocketAddr,
}

pub fn main(cfg: &Config) {
    println!("Remote address: {:?}", cfg.remote_addr);
}
