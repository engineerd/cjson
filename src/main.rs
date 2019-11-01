extern crate cjson;
extern crate serde_json;

use std::{env, fs, path};

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = &args[1];
    let input = fs::File::open(path::Path::new(input)).expect("cannot open input file");
    let res: serde_json::Value =
        serde_json::from_reader(input).expect("cannot deserialize input file");

    println!(
        "{}",
        cjson::to_string(&res).expect("cannot write canonical JSON")
    );
}
