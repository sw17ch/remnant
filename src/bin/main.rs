extern crate clap;
extern crate remnant;
extern crate serde_json;

use clap::{App,Arg,SubCommand};
use remnant::plan;
use remnant::remnant::Remnant;

fn main() {
    let matches = App::new("remnant")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(Arg::with_name("no-validate")
             .help("don't validate node identifiers")
             .required(false)
             .short("n"))
        .arg(Arg::with_name("path")
             .help("path to remnant database")
             .required(false)
             .value_name("PATH")
             .takes_value(true)
             .short("p"))
        .subcommand(SubCommand::with_name("append")
                    .about("adds a new record")
                    .arg(Arg::with_name("parent")
                         .help("the identifier for the parent record")
                         .value_name("PARENT")
                         .required(true)
                         .takes_value(true))
                    .arg(Arg::with_name("body")
                         .help("the content to append")
                         .value_name("BODY")
                         .required(false)
                         .takes_value(true)))
        .subcommand(SubCommand::with_name("origin")
                    .about("create a new timeline origin")
                    .arg(Arg::with_name("name")
                         .help("the origin's name")
                         .value_name("NAME")
                         .required(true)))
        .subcommand(SubCommand::with_name("join")
                    .about("create a new record joining two hashes together")
                    .arg(Arg::with_name("left")
                         .required(true)
                         .value_name("LEFT")
                         .takes_value(true))
                    .arg(Arg::with_name("right")
                         .required(true)
                         .value_name("RIGHT")
                         .takes_value(true)))
        .get_matches();

    plan::get_plan(&matches)
        .map(|mut p| run_plan(&mut p))
        .map_err(|e| println!("error: {}", e))
        .unwrap_or(());
}

fn run_plan(plan: &mut plan::Plan) {
    println!("plan: {:?}", plan);

    let r = match &plan.command {
        &plan::Command::Append { parent: ref p, body: ref b} => mk_valid_append(plan, p, b),
        &plan::Command::Origin { name: ref n } => mk_valid_origin(plan, n),
        &plan::Command::Join { left: ref l, right: ref r } => mk_valid_join(plan, l, r),
    };

    println!("remnant: {:?}", r);

    plan.database.insert(&r).unwrap();
}

fn mk_valid_append(_plan: &plan::Plan, _parent: &str, _body: &[u8]) -> Remnant {
    panic!("mk_valid_append")
}

fn mk_valid_origin(plan: &plan::Plan, name: &str) -> Remnant {
    Remnant::origin(&plan.author, name)
}

fn mk_valid_join(_plan: &plan::Plan, _left: &str, _right: &str) -> Remnant {
    panic!("mk_valid_join")
}
