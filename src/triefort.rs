use std::default;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use bincode;
use serde;
use serde_json;

const DEFAULT_LEVELS: usize = 2;

/// Trieforts are contained in a parent directory.

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    /// Levels the number of levels to use. This is how many
    /// sub-directories will be created.
    levels: usize,
}

impl Config {
    fn min_key_size(&self) -> usize {
        1 + self.levels
    }

    fn dir_from_key(&self, key: &[u8]) -> PathBuf {
        let mut p = PathBuf::new();

        for c in key.chunks(1).take(self.levels) {
            let hex = to_hex(c);
            p.push(hex);
        }

        p
    }
}

impl default::Default for Config {
    fn default() -> Self {
        Config {
            levels: DEFAULT_LEVELS,
        }
    }
}

#[derive(Debug)]
pub struct Handle<T> {
    cfg: Config,
    root: String,
    _phantom: PhantomData<T>,
}

pub fn open<T: Triefort>(path: &str) -> Result<Handle<T>, io::Error> {
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
        _phantom: PhantomData,
    })
}

fn to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);

    for b in bytes {
        hex.push_str(&format!("{:02x}", b));
    }

    hex
}

fn err<T>(msg: &str) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::Other, msg))
}

fn add_files(root: &PathBuf, key: &[u8], paths: &mut Vec<String>) {
    let hex = to_hex(key);
    let _ = fs::read_dir(root).map(|rd| {
        for d in rd {
            let _ = d.map(|p| {
                let path = p.path();
                if path.is_dir() {
                    add_files(&root.join(&path), key, paths);
                } else {
                    let filename = path.as_path().file_name().unwrap().to_str().unwrap();
                    let c = &filename[0..hex.len()];

                    if c == hex {
                        paths.push(filename.to_string());
                    }
                }
            });
        }
    });
}

fn files_matching<'a, T>(hdl: &'a Handle<T>, key: &'a [u8]) -> io::Result<Vec<String>> {
    let mut root = PathBuf::new();
    root.push(&hdl.root);

    for c in key.chunks(1).take(hdl.cfg.levels) {
        root.push(&to_hex(c));
    }

    let mut file_paths: Vec<String> = Vec::new();
    add_files(&root, key, &mut file_paths);

    Ok(file_paths)
}

impl<T: Triefort> Handle<T> {
    pub fn insert(&mut self, item: &T) -> io::Result<()> {
        let k = item.key();

        if k.len() < self.cfg.min_key_size() {
            panic!("Key must have a size of at least {} bytes.", k.len());
        }

        // There's an implicit maximum key length based on the file
        // system. We should probably lock that down at some point so
        // that trieforts are transferrable.

        let dir_path = Path::new(&self.root).join(self.cfg.dir_from_key(k));
        let item_path = dir_path.join(to_hex(k));

        if item_path.exists() {
            err("Item already exists.")
        } else {
            fs::create_dir_all(dir_path)?;
            let mut f = fs::File::create(item_path)?;
            f.write_all(&item.encode())?;
            Ok(())
        }
    }

    pub fn get(&mut self, key: &[u8]) -> io::Result<T> {
        let i: T = self.get_unchecked(key)?;
        if !i.check(key) {
            err("Item failed check.")
        } else {
            Ok(i)
        }
    }

    pub fn get_unchecked(&mut self, key: &[u8]) -> io::Result<T> {
        let p = Path::new(&self.root)
            .join(self.cfg.dir_from_key(key))
            .join(to_hex(key));

        if p.exists() {
            let mut v = Vec::new();
            let mut fh = fs::File::open(p)?;
            let _ = fh.read_to_end(&mut v).unwrap();
            Ok(T::decode(&v))
        } else {
            err("Item not in triefort.")
        }
    }

    pub fn find_all_with_prefix<'a>(&'a mut self, key: &'a [u8]) -> io::Result<Vec<String>> {
        files_matching(self, key)
    }
}

pub trait Triefort
where
    Self: serde::Serialize + serde::de::DeserializeOwned,
{
    fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
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
    }

    impl Triefort for Thing {
        fn key(&self) -> &[u8] {
            &self.key[..]
        }
    }

    #[test]
    fn it_works() {
        let tdir = tempdir::TempDir::new("triefort_test").unwrap();
        let mut hdl = open::<Thing>(tdir.path().to_str().unwrap()).unwrap();
        println!("hdl: {:?}", hdl);

        let t1_key = vec![1, 2, 3, 4];
        let t2_key = vec![5, 6, 7, 8];

        let t1 = Thing {
            key: t1_key.clone(),
        };
        let t2 = Thing {
            key: t2_key.clone(),
        };

        // First we check that we can insert things and that they end
        // up at the right path.
        hdl.insert(&t1).unwrap();
        hdl.insert(&t2).unwrap();

        let t1_path = tdir.path().join("01").join("02").join("01020304");
        let t2_path = tdir.path().join("05").join("06").join("05060708");

        assert!(t1_path.exists());
        assert!(t2_path.exists());

        // Now we're going to see that we can read things out and that
        // they match the original keys.
        assert_eq!(t1, hdl.get(&t1_key).unwrap());
        assert_eq!(t2, hdl.get(&t2_key).unwrap());
    }

    #[test]
    fn find_all_finds_all_files() {
        let tdir = tempdir::TempDir::new("triefort_test").unwrap();
        let mut hdl = open::<Thing>(tdir.path().to_str().unwrap()).unwrap();

        let t1 = Thing {
            key: vec![1, 2, 3, 4, 5],
        };
        let t2 = Thing {
            key: vec![1, 5, 6, 0, 0],
        };
        let t3 = Thing {
            key: vec![2, 8, 9, 0, 0],
        };
        let t4 = Thing {
            key: vec![1, 2, 3, 9, 0],
        };

        hdl.insert(&t1).unwrap();
        hdl.insert(&t2).unwrap();
        hdl.insert(&t3).unwrap();
        hdl.insert(&t4).unwrap();

        let found = hdl.find_all_with_prefix(&[1, 2, 3, 4]).unwrap();

        assert_eq!(vec!["0102030405"], found);
    }
}
