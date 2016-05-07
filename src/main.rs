extern crate remnant;

fn main() {
    let mut r = remnant::RemnantDB::new();
    let mut anchor = r.create_str("playground");
    let root = anchor.clone();

    anchor = r.append_str(&anchor, "this is a much longer string that should be truncated a bit when displayed");

    for i in 0..4 {
        let s = format!("append {}", i);
        anchor = r.append_str(&anchor, &s);
    }

    r.join(&root, &anchor);

    for v in r.iter() {
        println!("{}", v);
    }
}
