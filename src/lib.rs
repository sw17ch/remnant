extern crate sodiumoxide;

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

mod util;
