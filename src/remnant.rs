use sodiumoxide::crypto::hash::sha256 as hash;
use sodiumoxide::crypto::sign;
use std::fmt;
use author::{Author, AuthorId};
use util;
use ::triefort;


/// The primary storage container for all nodes in a Remnant database.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Remnant {
    /// The ID of the node. In this implementation, it's a SHA256 of
    /// the author and the content.
    id: NodeId,

    /// The ID of the author. Typically this is the fingerprint
    /// associated with the Author's key pair.
    author: AuthorId,

    /// The inner content of the Remnant.
    content: Content,

    /// The signature of the Remnant. The signature is generated using
    /// the node, the author, and the content. It is to give
    /// confidence in the message integrity and that the specified
    /// author was the source of the message.
    signature: Signature,
}

fn remnant_id_and_sig(author: &Author, content: &Content) -> (NodeId, Signature) {
    let mut hasher = hash::State::new();

    hasher.update(author.id().bytes());
    hasher.update(content.bytes().as_slice());

    let nodeid = NodeId(hasher.finalize());
    let sig = sign::sign_detached(&nodeid.bytes(), &author.sk());

    (nodeid, Signature(sig))
}

/// Create a remnant from an Author and a Content.
pub fn build_remnant(author: &Author, content: Content) -> Remnant {
    let (nodeid, sig) = remnant_id_and_sig(author, &content);

    Remnant {
        id: nodeid,
        author: author.id().clone(),
        content: content,
        signature: sig,
    }
}

/// Create a Remannt from discrete parts. This returns an *unchecked*
/// object that may not be valid.
pub fn build_remnant_from_parts(
    id: NodeId,
    author: AuthorId,
    content: Content,
    signature: Signature,
) -> Remnant {
    Remnant {
        id: id,
        author: author,
        content: content,
        signature: signature,
    }

}

/// Describes the sort of validation issues encountered when
/// validating a Remnant.
#[derive(Debug, PartialEq, Eq)]
pub enum ValidationErr {
    /// The Remnants `AuthorId` (left) and the provided `AuthorId` (right)
    AuthorMismatch(AuthorId, AuthorId),

    /// The Remnants `NodeId` (left) and the computed `NodeId` (right)
    IdentifierMismatch(NodeId, NodeId),

    /// The Remnants `Signature` (left) and the computed `Signature` (right)
    SignatureMismatch(Signature, Signature),
}


impl Remnant {
    /// The id of the Remnant
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    /// The author of the Remnant
    pub fn author(&self) -> &AuthorId {
        &self.author
    }

    /// The content of the Remnant
    pub fn content(&self) -> &Content {
        &self.content
    }

    /// The signature of the Remnant
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Create a new Origin.
    pub fn origin(author: &Author, name: &str) -> Remnant {
        let c = Content::Origin { name: name.to_string() };
        build_remnant(author, c)
    }

    /// Create a new Append after this one.
    pub fn append(&self, author: &Author, body: &[u8]) -> Remnant {
        let c = Content::Append {
            parent: self.id.clone(),
            body: Body(body.to_vec()),
        };
        build_remnant(author, c)
    }

    /// Create a new Join referencing these two.
    pub fn join(author: &Author, left: &Remnant, right: &Remnant) -> Remnant {
        let c = Content::Join {
            left: left.id.clone(),
            right: right.id.clone(),
        };
        build_remnant(author, c)
    }

    /// Check that the Remnant is valid. The ID and the Signature
    /// should match the other contents in the Remannt.
    pub fn validate(&self, author: &Author) -> Result<(), ValidationErr> {
        let (id, sig) = remnant_id_and_sig(author, &self.content);

        if *author.id() != self.author {
            Err(ValidationErr::AuthorMismatch(
                self.author.clone(),
                author.id().clone(),
            ))
        } else if id != self.id {
            Err(ValidationErr::IdentifierMismatch(self.id.clone(), id))
        } else if sig != self.signature {
            Err(ValidationErr::SignatureMismatch(
                self.signature.clone(),
                sig,
            ))
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for Remnant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Remnant(id: {}, author: {}, signature: {})",
               self.id,
               self.author,
               self.signature,
        )
    }
}

impl triefort::Triefort for Remnant {
    fn key(&self) -> &[u8] {
        self.id.bytes()
    }
}


/// An identifier for a node that should be unique for a given
/// timeline. This implementation uses a SHA256 for the Node ID.
#[derive(PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct NodeId(pub hash::Digest);

impl NodeId {
    pub fn bytes(&self) -> &[u8] {
        let &NodeId(hash::Digest(ref bytes)) = self;
        bytes
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &NodeId(hash::Digest(ref bytes)) = self;

        write!(f, "NodeId")?;
        util::debug_bytes(f, bytes)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &NodeId(hash::Digest(ref bytes)) = self;
        util::display_bytes(f, bytes)
    }
}


/// The content variation allowed inside a Remnant.
#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum Content {
    /// The start of a Remnant timeline. It's just a string
    /// identifying the origin.
    Origin { name: String },

    /// Appends new data to a Remnant timeline. It specifies the
    /// parent preceeding the node and the body of the node.
    Append { parent: NodeId, body: Body },

    /// Appends a record that specifies two nodes as parents. This
    /// gives confidence that children of this node follow both of
    /// this node's parents. More than two nodes can be joined by
    /// chaining/folding joins.
    Join { left: NodeId, right: NodeId },
}

impl fmt::Debug for Content {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Content::Origin { name: ref n } => write!(f, "Content::Origin {{ name: {:?} }}", n),
            &Content::Append {
                parent: ref p,
                body: ref b,
            } => write!(f, "Content::Append {{ parent: {:?}, body: {:?} }}", p, b),
            &Content::Join {
                left: ref l,
                right: ref r,
            } => write!(f, "Content::Join( {{ left: {:?}, right: {:?} }}", l, r),
        }
    }
}

/// A node body used with an Append. It is an arbitrary array of
/// bytes.
#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct Body(pub Vec<u8>);

impl Body {
    pub fn bytes(&self) -> &[u8] {
        let &Body(ref v) = self;
        &v[..]
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Body(ref vec) = self;
        write!(f, "Body({:?})", String::from_utf8_lossy(vec))
    }
}

/// A signature for the message.
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Signature(sign::Signature);

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Signature(sign::Signature(ref bytes)) = self;
        write!(f, "Signature")?;
        util::debug_bytes(f, bytes)
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Signature(sign::Signature(ref bytes)) = self;
        util::display_bytes(f, bytes)
    }
}

impl Content {
    fn bytes(&self) -> Vec<u8> {
        match self {
            &Content::Origin { name: ref n } => n.as_bytes().to_vec(),
            &Content::Append {
                parent: ref p,
                body: ref b,
            } => {
                let mut vec = p.bytes().to_vec();
                vec.extend(b.bytes());
                vec
            }
            &Content::Join {
                left: ref l,
                right: ref r,
            } => {
                let mut vec = l.bytes().to_vec();
                vec.extend(r.bytes());
                vec
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let author = Author::new();
        let name = "hello world".to_string();

        let mut hasher = hash::State::new();
        hasher.update(author.id().bytes());
        hasher.update(name.as_bytes());
        let node_id = NodeId(hasher.finalize());

        let sig = Signature(sign::sign_detached(node_id.bytes(), &author.sk()));

        let expected = Remnant {
            id: node_id,
            author: author.id().clone(),
            content: Content::Origin { name: name.clone() },
            signature: sig,
        };

        assert_eq!(expected, Remnant::origin(&author, &name));
    }

    #[test]
    fn different_authors_have_different_remnants() {
        let a1 = Author::new();
        let a2 = Author::new();

        assert_ne!(a1, a2);

        let r1 = build_remnant(&a1, Content::Origin { name: "now what".to_string() });
        let r2 = build_remnant(&a2, Content::Origin { name: "now what".to_string() });

        assert_ne!(r1, r2);

        let ra1 = r1.append(&a2, b"this is new");
        let ra2 = r2.append(&a1, b"this is also new");

        let j = Remnant::join(&a1, &ra1, &ra2);
        println!("j: {}", j);

        j.validate(&a1).unwrap();
    }
}
