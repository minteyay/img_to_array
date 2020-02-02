extern crate getopts;
use getopts::Options;
use std::env;
use std::process;

use img_to_array::Config;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} PALETTE_PATH IMAGE_PATH [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    // Setup getopts options and flags
    let mut opts = Options::new();
    opts.optopt("o", "output", "set output file name (output.c by default)",
        "NAME");
    opts.optflag("h", "help", "print this help message");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => {
            eprintln!("{}", err.to_string());
            print_usage(&program, opts);
            process::exit(1);
        }
    };

    // Check if the help menu was requested
    if matches.opt_present("h") {
        print_usage(&program, opts);
        process::exit(1);
    }

    // Check that the correct number of inputs were given
    if matches.free.len() != 2 {
        println!("No palette and image files specified");
        print_usage(&program, opts);
        process::exit(1);
    }

    // Set the output path to one specified or a default one
    let output_path = match matches.opt_str("o") {
        Some(v) => v,
        None => String::from("output.c"),
    };

    let palette_path = matches.free[0].clone();
    let image_path = matches.free[1].clone();

    // Do the actual conversion
    let config = Config { palette_path, image_path, output_path };
    if let Err(e) = img_to_array::convert(&config) {
        println!("{}", e);
        process::exit(1);
    };
    println!("Arrays written successfully to file \"{}\"", config.output_path);
}
