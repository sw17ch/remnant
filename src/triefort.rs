use std::fs;
use std::path::{Path,PathBuf};
use std::io;
use std::io::{Read,Write};
use std::default;
use bincode;

use serde;
use serde_json;

const DEFAULT_WIDTH: usize = 1;
const DEFAULT_LEVELS: usize = 2;

/// Trieforts are contained in a parent directory.

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    /// The number of bytes to use at each level.
    width: usize,

    /// Levels the number of levels to use. This is how many
    /// sub-directories will be created.
    levels: usize,
}

impl Config {
    fn min_key_size(&self) -> usize {
        1 + (self.width * self.levels)
    }

    fn dir_from_key(&self, key: &[u8]) -> PathBuf {
        let mut p = PathBuf::new();

        for c in key.chunks(self.width).take(self.levels) {
            let hex = to_hex(c);
            p.push(hex);
        }

        p
    }
}

impl default::Default for Config {
    fn default() -> Self {
        Config {
            width: DEFAULT_WIDTH,
            levels: DEFAULT_LEVELS,
        }
    }
}

#[derive(Debug)]
pub struct Handle {
    cfg: Config,
    root: String,
}

pub fn open(path: &str) -> Result<Handle, io::Error> {
    let p = Path::new(path);
    let p_cfg = p.join("config.json");

    let cfg = if p.exists() && p_cfg.exists() {
        let mut f_cfg = fs::File::open(p_cfg)?;
        let mut s_cfg = String::new();
        f_cfg.read_to_string(&mut s_cfg)?;
        serde_json::from_str(&s_cfg)?
    } else {
        fs::create_dir_all(p)?;

        let default_cfg = Config::default();
        let default_cfg_json = serde_json::to_string_pretty(&default_cfg).unwrap();
        let mut f_cfg = fs::File::create(&p_cfg)?;
        f_cfg.write_all(default_cfg_json.as_bytes())?;

        default_cfg
    };

    Ok(Handle {
        cfg: cfg,
        root: path.to_string(),
    })
}

fn to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);

    for b in bytes {
        hex.push_str(&format!("{:02x}", b));
    }

    hex
}

fn err<T>(msg: &str) -> Result<T, io::Error> {
    Err(io::Error::new(io::ErrorKind::Other, msg))
}

impl Handle {
    pub fn insert<T: Triefort>(&mut self, item: &T) -> io::Result<()> {
        let k = item.key();

        if k.len() < self.cfg.min_key_size() {
            panic!("Key must have a size of at least {} bytes.", k.len());
        }

        // There's an implicit maximum key length based on the file
        // system. We should probably lock that down at some point so
        // that trieforts are transferrable.

        let dir_path = Path::new(&self.root)
            .join(self.cfg.dir_from_key(k));
        let item_path = dir_path
            .join(to_hex(k));

        if item_path.exists() {
            err("Item already exists.")
        } else {
            fs::create_dir_all(dir_path)?;
            let mut f = fs::File::create(item_path)?;
            f.write_all(&item.encode())?;
            Ok(())
        }
    }

    pub fn get<T: Triefort>(&mut self, key: &[u8]) -> io::Result<T> {
        let i: T = self.get_unchecked(key)?;
        if !i.check(key) {
            err("Item failed check.")
        } else {
            Ok(i)
        }
    }

    pub fn get_unchecked<T: Triefort>(&mut self, key: &[u8]) -> io::Result<T> {
        let p = Path::new(&self.root)
            .join(self.cfg.dir_from_key(key))
            .join(to_hex(key));

        if p.exists() {
            let mut v = Vec::new();
            let mut fh = fs::File::open(p)?;
            let size = fh.read_to_end(&mut v).unwrap();
            println!("read {} bytes", size);
            Ok(T::decode(&v))
        } else {
            err("Item not in triefort.")
        }
    }
}

pub trait Triefort
    where Self: serde::Serialize + serde::de::DeserializeOwned {

    fn encode(&self) -> Vec<u8> {
        bincode::serialize(self, bincode::Infinite).unwrap()
    }

    fn decode(enc: &[u8]) -> Self {
        bincode::deserialize(enc).unwrap()
    }

    fn check(&self, key: &[u8]) -> bool {
        self.key() == key
    }

    fn key(&self) -> &[u8];
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir;

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct Thing {
        key: Vec<u8>,
        value: String,
    }

    impl Triefort for Thing {
        fn key(&self) -> &[u8] {
            &self.key[..]
        }
    }

    #[test]
    fn it_works() {
        let tdir = tempdir::TempDir::new("triefort_test").unwrap();
        let mut hdl = open(tdir.path().to_str().unwrap()).unwrap();
        println!("hdl: {:?}", hdl);

        let t1_key = vec![1,2,3,4];
        let t2_key = vec![5,6,7,8];

        let t1 = Thing { key: t1_key.clone(), value: "Thing One".to_string() };
        let t2 = Thing { key: t2_key.clone(), value: "Thing Two".to_string() };

        // First we check that we can insert things and that they end
        // up at the right path.
        hdl.insert(&t1).unwrap();
        hdl.insert(&t2).unwrap();

        let t1_path = tdir.path()
            .join("01")
            .join("02")
            .join("01020304");
        let t2_path = tdir.path()
            .join("05")
            .join("06")
            .join("05060708");

        assert!(t1_path.exists());
        assert!(t2_path.exists());

        // Now we're going to see that we can read things out and that
        // they match the original keys.
        assert_eq!(t1, hdl.get(&t1_key).unwrap());
        assert_eq!(t2, hdl.get(&t2_key).unwrap());
    }
}
