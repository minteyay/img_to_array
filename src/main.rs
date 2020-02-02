use std::env;
use std::process;

use img_to_array::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else( |err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    img_to_array::convert(config);
}
