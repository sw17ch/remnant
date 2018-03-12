extern crate clap;
use clap::ArgMatches;
use std::env::home_dir;
use std::io;
use std::io::{Read, Write};
use author::Author;
use triefort;
use remnant::Remnant;
use std::path::{Path, PathBuf};
use std::fs;

use serde_json;

#[derive(Debug)]
pub enum Command {
    Append { parent: String, body: Vec<u8> },
    Origin { name: String },
    Join { left: String, right: String },
}

#[derive(Debug)]
pub struct Plan {
    pub validate: bool,
    pub path: String,
    pub command: Command,

    pub author: Author,
    pub database: triefort::Handle<Remnant>,
}

pub fn get_plan(a: &ArgMatches) -> io::Result<Plan> {
    let path = if a.is_present("path") {
        a.value_of("path").unwrap().to_string()
    } else {
        let mut h = home_dir().unwrap_or(PathBuf::from("/"));
        h.push(".remnant");
        h.to_str().unwrap().to_string()
    };

    let author = get_author(&path)?;
    let database = get_database(&path)?;

    match a.subcommand() {
        ("append", Some(a)) => cmd_append(a),
        ("origin", Some(o)) => cmd_origin(o),
        ("join", Some(j)) => cmd_join(j),
        (c, _) => err(&format!("unexpected subcommand: {}", c)),
    }.map(|c| Plan {
        validate: !a.is_present("no-validate"),
        path: path,
        command: c,
        author: author,
        database: database,
    })
}

fn get_author(path: &str) -> io::Result<Author> {
    fs::create_dir_all(path)?;

    let a_path = Path::new(path).join("author.json");

    let a = if !a_path.exists() {
        let a = Author::new();
        let a_json = serde_json::to_string(&a)?;
        let mut a_f = fs::File::create(&a_path)?;
        a_f.write_all(a_json.as_bytes())?;

        a
    } else {
        let mut a_f = fs::File::open(&a_path)?;
        let mut a_str = String::new();
        a_f.read_to_string(&mut a_str)?;
        serde_json::from_str(&a_str)?
    };

    Ok(a)
}

fn get_database(path: &str) -> io::Result<triefort::Handle<Remnant>> {
    fs::create_dir_all(path)?;

    let triefort_path = Path::new(path).join("database");
    let db = triefort::open(triefort_path.to_str().unwrap())?;

    Ok(db)
}

fn cmd_append(a: &ArgMatches) -> io::Result<Command> {
    let op = a.value_of("parent");
    let ob = a.value_of("body");

    match (op, ob) {
        (Some(p), Some(b)) => Ok(Command::Append {
            parent: p.to_string(),
            body: b.as_bytes().to_vec(),
        }),
        (None, _) => err("bad parent"),
        (_, None) => err("bad body"),
    }
}

fn cmd_origin(a: &ArgMatches) -> io::Result<Command> {
    let on = a.value_of("name");

    match on {
        Some(n) => Ok(Command::Origin {
            name: n.to_string(),
        }),
        None => err("bad name"),
    }
}

fn cmd_join(a: &ArgMatches) -> io::Result<Command> {
    let ol = a.value_of("left");
    let or = a.value_of("right");

    match (ol, or) {
        (Some(l), Some(r)) => Ok(Command::Join {
            left: l.to_string(),
            right: r.to_string(),
        }),
        (None, _) => err("bad left"),
        (_, None) => err("bad right"),
    }
}

fn err<T>(msg: &str) -> Result<T, io::Error> {
    Err(io::Error::new(io::ErrorKind::Other, msg))
}
