use std::env;
use std::process;

fn main() {
    // Parse command line arguments into a config
    let config = img_to_array::parse_config(env::args().collect())
        .unwrap_or_else( |_| { process::exit(1); });

    // Convert the file
    if let Err(e) = img_to_array::convert(&config) {
        println!("{}", e);
        process::exit(1);
    };
    println!("Arrays written successfully to file \"{}\"", config.output_path);
}
