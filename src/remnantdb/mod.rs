extern crate crypto;
extern crate uuid;

use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;
use self::uuid::Uuid;

use std::collections::hash_map::{HashMap,Values};
use std::fmt;
use std::cmp::{min};
pub type Name = Vec<u8>;

/// UUIDs are used to uniquely identify a timeline. To generate this,
/// the host machine needs to have a reasonably strong source of
/// randomness.
#[derive(Debug)]
pub struct UUID {
    bytes: [u8;16]
}

/// Anchors are a hash of the event's data. They are used to uniquely
/// identify an event.
#[derive(Debug,PartialEq,Eq,Hash,Clone,Copy)]
pub struct Anchor {
    bytes: [u8;32]
}

/// Payloads are a sum of timeline heads, appending events, and joins.
#[derive(Debug)]
pub enum Payload {
    /// A timeline payload acts as the start of a timeline. It does
    /// not refer to other events and comes before all other events in
    /// the timeline. The anchor for a timeline is derived from both
    /// the name and the UUID. This allows us to have distinct anchors
    /// even for timelines with the same name.
    Timeline {
        /// A name for the timeline.
        name: Name,

        /// A universally unique identifier for the timeline.
        uuid: UUID,
    },

    /// Append events allow new data to be concatenated into a
    /// timeline.
    Append {
        /// The anchor of this event's ancestor.
        ancestor: Anchor,

        /// The data payload for this event.
        payload: Vec<u8>,
    },

    /// A join allows to events to be marked as predecessors of the
    /// join event. All events that follow the join are then known to
    /// follow the left and right events of the join.
    Join {
        /// One of the events in the join.
        left: Anchor,

        /// One of the events in the join.
        right: Anchor,
    },
}

/// An anchor and a payload. The anchor is a hash of the data in the
/// payload.
#[derive(Debug)]
pub struct Event {
    anchor: Anchor,
    payload: Payload,
}

///A map from Anchors to Events.
#[derive(Debug)]
pub struct RemnantDB {
    events: HashMap<Anchor, Event>,
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

    pub fn len(&self) -> usize {
        self.events.len()
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
        let s = slice_to_hex_string(&self.bytes[0..4]);
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

                format!("Timeline ({} is {})", c, u)
            },
            &Payload::Append {ancestor: ref a, payload: ref p} => {
                let s = String::from_utf8((*p).clone());
                match s {
                    Ok(valid_str) => {
                        let rng = min(12, valid_str.len());
                        format!("Append ({} <- \"{}\")", a, &valid_str[0..rng])
                    }
                    Err(_) => {
                        let rng = min(12, p.len());
                        format!("Append ({} <- {})", a, slice_to_hex_string(&p[0..rng]))
                    },
                }
            },
            &Payload::Join {left: ref l, right: ref r} => {
                format!("Join ({} + {})", l, r)
            },
        };

        write!(f, "{}", s)
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Event { anchor: ref a, payload: ref b } = self;

        write!(f, "({} -> {})", a, b)
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
