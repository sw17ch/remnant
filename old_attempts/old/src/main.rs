extern crate remnant;

use remnant::remnantdb;
use std::io;
use std::io::prelude::*;

fn prompt(a: &remnant::remnantdb::Anchor) {
    print!("{}$ ", a);
    let _ = io::stdout().flush();
}

fn help() {
    println!("exit - quit");
    println!("help - list this help information");
    println!("list - list all entries in the database");
}

fn main() {
    let mut r = remnantdb::RemnantDB::new();

    let root = r.create_str("console");
    let mut anchor = root;

    let stdin = io::stdin();

    prompt(&anchor);

    for line in stdin.lock().lines() {
        let u = line.unwrap();

        match (&u).trim() {
            "help" => help(),
            "exit" => break,
            "list" => {
                for e in r.iter() {
                    println!("{}", e);
                }
            },
            "" => {},
            s => {
                anchor = r.append_str(&anchor, s);
                println!("! {}", r.len());
            },
        }
        prompt(&anchor);
    }
}
