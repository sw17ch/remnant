extern crate crypto;
extern crate uuid;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use uuid::Uuid;

use std::collections::hash_map::{HashMap};

pub type Name = Vec<u8>;
pub type GUID = [u8;16];
pub type Anchor = [u8;32];

pub enum Payload {
    Timeline {
        name: Name,
        guid: GUID,
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

pub struct Event {
    anchor: Anchor,
    payload: Payload,
}

pub struct RemnantDB {
    events: HashMap<Anchor, Event>,
}

impl Payload {
    fn anchor(&self) -> Anchor {
        let mut hasher = Sha256::new();
        let mut result: Anchor = [0;32];

        match self {
            &Payload::Timeline { name: ref n, guid: ref g } => {
                hasher.input(n);
                hasher.input(g);
            },
            &Payload::Append { ancestor: ref a, payload: ref p } => {
                hasher.input(a);
                hasher.input(p);
            },
            &Payload::Join { left: ref l , right: ref r } => {
                hasher.input(l);
                hasher.input(r);
            },
        }

        hasher.result(&mut result);

        result
    }
}

impl Event {
    pub fn anchor(&self) -> &[u8] {
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
            guid: *Uuid::new_v4().as_bytes(),
        };
        let a = payload.anchor();

        let e = Event {
            anchor: a.clone(),
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
            ancestor: ancestor.clone(),
            payload: payload.to_vec(),
        };
        let a = payload.anchor();

        let e = Event {
            anchor: a.clone(),
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
            left: left.clone(),
            right: right.clone(),
        };
        let a = payload.anchor();

        let e = Event {
            anchor: a.clone(),
            payload: payload,
        };

        self.events.insert(a, e);

        a
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_can_create_timelines() {
        let mut r = ::RemnantDB::new();
        let timeline = r.create_str("test");
        let append_1 = r.append(&timeline, "append 1".as_bytes());
        let append_2 = r.append(&timeline, "append 2".as_bytes());
        r.join(&append_1, &append_2);
    }
}
