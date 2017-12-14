use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::hash::sha256 as hash;
use std::fmt;
use ::util;

/// A description of an author. Contains the public and private key
/// along with a hash of the public key.
#[derive(Debug, PartialEq, Eq)]
pub struct Author {
    pk: sign::PublicKey,
    sk: sign::SecretKey,
    id: AuthorId,
}

impl Author {
    pub fn new() -> Author {
        let id: AuthorId;
        let (pk, sk) = sign::gen_keypair();
        let &sign::PublicKey(ref pk_bytes) = &pk;
        id = AuthorId(hash::hash(pk_bytes));

        Author {
            pk: pk,
            sk: sk,
            id: id,
        }
    }

    pub fn id(&self) -> &AuthorId {
        &self.id
    }

    pub fn sk(&self) -> &sign::SecretKey {
        &self.sk
    }

    pub fn pk(&self) -> &sign::PublicKey {
        &self.pk
    }
}

/// An author ID. This is a hash of the Author's public key.
#[derive(PartialEq, Eq, Clone)]
pub struct AuthorId(pub hash::Digest);

impl AuthorId {
    pub fn bytes(&self) -> &[u8] {
        let &AuthorId(hash::Digest(ref bytes)) = self;
        bytes
    }
}

impl fmt::Debug for AuthorId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &AuthorId(hash::Digest(ref bytes)) = self;

        write!(f, "AuthorId")?;
        util::debug_bytes(f, bytes)
    }
}

impl fmt::Display for AuthorId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &AuthorId(hash::Digest(ref bytes)) = self;
        util::display_bytes(f, bytes)
    }
}
