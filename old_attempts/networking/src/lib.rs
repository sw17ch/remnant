#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use] extern crate log;
extern crate env_logger;

extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_io;
extern crate tokio_service;
extern crate futures;
extern crate bytes;
extern crate bincode;
extern crate byteorder;

pub mod client;
pub mod server;
pub mod message;

use std::io;
use std::mem::size_of;
use std::fmt::Debug;
use std::io::Cursor;

use serde::de::DeserializeOwned;
use serde::Serialize;
use bincode::{serialize, deserialize, Infinite};
use bytes::{BufMut, BytesMut, BigEndian};
use byteorder::ReadBytesExt;
use byteorder::BigEndian as BOBigEndian;

pub fn wrap_size<T>(item: &T, dst: &mut BytesMut) -> Result<(), io::Error>
    where T: Serialize + Debug
{
    debug!("ENC: {:?}", item);

    let enc = serialize(&item, Infinite).unwrap();
    dst.put_u64::<BigEndian>(enc.len() as u64);
    dst.extend(enc);

    Ok(())
}

pub fn unwrap_size<T>(src: &mut BytesMut) -> Result<Option<T>, io::Error>
    where T: DeserializeOwned + Default + Debug
{
    if let Ok(len) = Cursor::new(&src).read_u64::<BOBigEndian>() {
        let header_size = size_of::<u64>();
        let message_len = header_size + len as usize;

        if src.len() < message_len {
            debug!("PENDING");
            Ok(None)
        } else {
            let msg = src.split_to(message_len);

            if header_size == message_len {
                debug!("EMPTY!");
                Ok(Some(Default::default()))
            } else {
                match deserialize(&msg[header_size..message_len]) {
                    Ok(resp) => {
                        debug!("DEC: {:?}", resp);
                        Ok(Some(resp))
                    }
                    Err(e) => {
                        error!("ERROR bad decode [{:?}]", e);
                        Err(io::Error::new(io::ErrorKind::Other, "bad decode"))
                    }
                }
            }
        }
    } else {
        Ok(None)
    }
}
