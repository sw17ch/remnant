extern crate crypto;
extern crate uuid;

use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;
use self::uuid::Uuid;

use std::collections::hash_map::{HashMap,Values};
use std::fmt;
use std::cmp::{min};
pub type Name = Vec<u8>;

#[derive(Debug)]
pub struct UUID {
    bytes: [u8;16]
}

#[derive(Debug,PartialEq,Eq,Hash,Clone,Copy)]
pub struct Anchor {
    bytes: [u8;32]
}

#[derive(Debug)]
pub enum Payload {
    Timeline {
        name: Name,
        uuid: UUID,
    },
    Append {
        ancestor: Anchor,
        payload: Vec<u8>,
    },
    Join {
        left: Anchor,
        right: Anchor,
    },
}

#[derive(Debug)]
pub struct Event {
    anchor: Anchor,
    payload: Payload,
}

#[derive(Debug)]
pub struct RemnantDB {
    events: HashMap<Anchor, Event>,
}

#[derive(Debug)]
pub struct AnchorRef<'a> {
    anchor: Anchor,
    db: &'a mut RemnantDB,
}

impl Payload {
    fn anchor(&self) -> Anchor {
        let mut hasher = Sha256::new();

        match self {
            &Payload::Timeline { name: ref n, uuid: ref g } => {
                hasher.input(n);
                hasher.input(&g.bytes);
            },
            &Payload::Append { ancestor: ref a, payload: ref p } => {
                hasher.input(&a.bytes);
                hasher.input(p);
            },
            &Payload::Join { left: ref l , right: ref r } => {
                hasher.input(&l.bytes);
                hasher.input(&r.bytes);
            },
        }

        let mut result = Anchor { bytes: [0;32] };

        hasher.result(&mut result.bytes);

        result
    }
}

impl Event {
    pub fn anchor(&self) -> &Anchor {
        &self.anchor
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}

impl RemnantDB {
    pub fn new() -> RemnantDB {
        RemnantDB { events: HashMap::new() }
    }

    pub fn create(&mut self, name: &[u8]) -> Anchor {
        let payload = Payload::Timeline {
            name: name.to_vec(),
            uuid: UUID { bytes: *Uuid::new_v4().as_bytes() },
        };
        let a = payload.anchor();

        let e = Event {
            anchor: a,
            payload: payload,
        };

        self.events.insert(a, e);

        a
    }

    pub fn create_str(&mut self, name: &str) -> Anchor {
        self.create(name.as_bytes())
    }

    pub fn append(&mut self, ancestor: &Anchor, payload: &[u8]) -> Anchor {
        let payload = Payload::Append {
            ancestor: *ancestor,
            payload: payload.to_vec(),
        };
        let a = payload.anchor();

        let e = Event {
            anchor: a,
            payload: payload,
        };

        self.events.insert(a, e);

        a
    }

    pub fn append_str(&mut self, ancestor: &Anchor, payload: &str) -> Anchor {
        self.append(ancestor, payload.as_bytes())
    }

    pub fn join(&mut self, left: &Anchor, right: &Anchor) -> Anchor {
        let payload = Payload::Join {
            left: *left,
            right: *right,
        };
        let a = payload.anchor();

        let e = Event {
            anchor: a,
            payload: payload,
        };

        self.events.insert(a, e);

        a
    }

    pub fn iter(&self) -> Values<Anchor, Event> {
        self.events.values()
    }
}

fn slice_to_hex_string(bytes: &[u8]) -> String {
    let strs: Vec<String> =
        bytes.iter().map(|b| format!("{:02x}", b)).collect();
    strs.concat()
}

impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let g = Uuid::from_bytes(&self.bytes).unwrap();
        write!(f, "{}", &g.hyphenated().to_string())
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = slice_to_hex_string(&self.bytes[0..8]);
        write!(f, "<{}>", &s)
    }
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            &Payload::Timeline {name: ref n, uuid: ref u} => {
                let s = String::from_utf8((*n).clone());
                let c = match s {
                    Ok(valid_str) => format!("\"{}\"", valid_str),
                    Err(_) => slice_to_hex_string(n),
                };

                format!("Timeline {} is {}", c, u)
            },
            &Payload::Append {ancestor: ref a, payload: ref p} => {
                let s = String::from_utf8((*p).clone());
                match s {
                    Ok(valid_str) => {
                        let rng = min(12, valid_str.len());
                        format!("Append str {} <- \"{}\"", a, &valid_str[0..rng])
                    }
                    Err(_) => {
                        let rng = min(12, p.len());
                        format!("Append data {} <- {}", a, slice_to_hex_string(&p[0..rng]))
                    },
                }
            },
            &Payload::Join {left: ref l, right: ref r} => {
                format!("Join {} + {}", l, r)
            },
        };

        write!(f, "Payload::{}", s)
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Event { anchor: ref a, payload: ref b } = self;

        write!(f, "({} => {})", a, b)
    }
}

impl fmt::Display for RemnantDB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(f.write_str("[RemnantDB"));

        for (_, ref event) in &self.events {
            try!(write!(f, " {}", event));
        }

        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_can_create_timelines() {
        let mut r = ::remnantdb::RemnantDB::new();
        let timeline = r.create_str("test");
        let append_1 = r.append(&timeline, "append 1".as_bytes());
        let append_2 = r.append(&timeline, "append 2".as_bytes());
        r.join(&append_1, &append_2);
    }

    #[test]
    fn equal_payloads_have_equal_anchors() {
        let p1 = ::remnantdb::Payload::Timeline {
            name: "t".as_bytes().to_vec(),
            uuid: ::remnantdb::UUID { bytes: [1;16] },
        };
        let p2 = ::remnantdb::Payload::Timeline {
            name: "t".as_bytes().to_vec(),
            uuid: ::remnantdb::UUID { bytes: [1;16] },
        };

        assert!(p1.anchor() == p2.anchor());
    }

    #[test]
    fn unequal_payloads_have_unequal_anchors() {
        let p1 = ::remnantdb::Payload::Timeline {
            name: "t".as_bytes().to_vec(),
            uuid: ::remnantdb::UUID { bytes: [1;16] },
        };
        let p2 = ::remnantdb::Payload::Timeline {
            name: "d".as_bytes().to_vec(),
            uuid: ::remnantdb::UUID { bytes: [1;16] },
        };

        assert!(p1.anchor() != p2.anchor());
    }
}
