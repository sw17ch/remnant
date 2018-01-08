#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate bincode;

extern crate tempdir;

extern crate sodiumoxide;
extern crate clap;

///! A remnant is the primary representation of items in the Remnant
///! system. It's an identifier, an author identifier, some content,
///! and a signature.
pub mod remnant;

///! An author is what creates Remnants. They consist of a public and
///! private key pair along with an identifier which is the hash of
///! the public key.
pub mod author;

///! A universe is a collection of Remnants. It supports methods for
///! inserting more Remnants, querying existing Remnants, and backing
///! the whole thing on disk.
pub mod universe;

///! A protocol is the set of messages we send and recieve from peers
///! in the network. These are typically meta messages about the
///! authors and universes on each node in the network.
pub mod protocol;

mod util;

///! An execution plan for the command line client.
pub mod plan;

///! Triefort is an on-disk trie that stores objects by hash in a
///! directory structure using tries.
pub mod triefort;
