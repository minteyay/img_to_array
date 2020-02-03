extern crate getopts;

use std::fs;
use getopts::Options;

#[derive(Debug)]
pub struct Config {
    pub image_path: String,
    pub palette_path: String,
    pub output_path: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Rgb565(u16);

impl From<(u8, u8, u8)> for Rgb565 {
    fn from(rgb: (u8, u8, u8)) -> Self {
        // First we shift to the right to truncate the values to the widths
        // we want (5 most significant bits from red, 6 from green, 5 from blue)
        // then shift them back to the left to their places in 565
        let r: u16 = u16::from(rgb.0) >> 3 << 11;
        let g: u16 = u16::from(rgb.1) >> 2 << 5;
        let b: u16 = u16::from(rgb.2) >> 3;
        Rgb565(r | g | b)
    }
}

impl From<&image::Bgra<u8>> for Rgb565 {
    fn from(bgra: &image::Bgra<u8>) -> Self {
        Rgb565(Rgb565::from((bgra[2], bgra[1], bgra[0])).0)
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} PALETTE_PATH IMAGE_PATH [options]", program);
    print!("{}", opts.usage(&brief));
}

pub fn parse_config(args: Vec<String>) -> Result<Config, ()> {
    // Get program name
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
            return Err(())
        }
    };

    // Check if the help menu was requested
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Err(())
    }

    // Check that the correct number of inputs were given
    if matches.free.len() != 2 {
        println!("No palette and image files specified");
        print_usage(&program, opts);
        return Err(())
    }

    // Set the output path to one specified or a default one
    let output_path = match matches.opt_str("o") {
        Some(v) => v,
        None => String::from("output.c"),
    };

    let palette_path = matches.free[0].clone();
    let image_path = matches.free[1].clone();

    Ok(Config { palette_path, image_path, output_path })
}

pub fn convert(config: &Config) -> Result<(), String> {
    let mut output = String::from("#include <stdint.h>\n");

    // Read in and store the palette
    let palette_img = match image::open(&config.palette_path) {
        Ok(img) => img,
        Err(e) => return Err(format!("Error opening palette file \"{}\": {}",
            &config.palette_path, e.to_string())),
    };
    let palette_img = palette_img.to_bgra();

    let mut palette: Vec<Rgb565> = Vec::new();

    // Add the colours from the palette to the vector
    for pixel in palette_img.enumerate_pixels() {
        let colour = Rgb565::from(pixel.2);
        palette.push(colour);
    }

    // Add the palette to the output
    output.push_str(format!("\nconst uint16_t palette[{}] PROGMEM = {{\n    ",
        palette.len()).as_str());
    let mut line = String::from("    ");
    for colour in &palette {
        // Check if we need to push the current value to the next line
        if line.len() + 7 > 80 {
            output.push_str(format!("{}\n", line).as_str());
            line = String::from("    ");
        }
        line.push_str(format!("{:#06X}, ", colour.0).as_str());
    }
    if !line.trim().is_empty() {
        output.push_str(line.as_str());
    }
    output.push_str("\n};\n\n");

    // Read in the image to convert and add its pixels to the output
    let img = match image::open(&config.image_path) {
        Ok(img) => img,
        Err(e) => return Err(format!("Error opening image file \"{}\": {}",
            &config.image_path, e.to_string())),
    };
    let img = img.to_bgra();

    // Add the image data array definition to the output
    output.push_str(format!("const uint8_t image_data[{}] PROGMEM = {{\n",
        img.dimensions().0 * img.dimensions().1).as_str());
    let mut line = String::from("    ");
    for pixel in img.enumerate_pixels() {
        // Convert the 32 bit RGBA pixel value to RGB565
        let colour = Rgb565::from(pixel.2);

        // Check if we need to push the current value to the next line
        if line.len() + 4 > 80 {
            output.push_str(format!("{}\n", line).as_str());
            line = String::from("    ");
        }

        // Check that the colour is defined in the palette
        let palette_index = match palette.iter().position( |c| c == &colour) {
            Some(v) => v,
            None => return Err(format!("Error creating colour index array: \
                colour {:#06X} isn't present in the palette", colour.0)),
        };
        line.push_str(format!("{:3},", palette_index).as_str());
    }
    if !line.trim().is_empty() {
        output.push_str(format!("{}\n", line).as_str());
    }
    output.push_str("};\n");

    // Write output to file
    if let Err(e) = fs::write(&config.output_path, output) {
        return Err(format!("Error writing output file: {}", e.to_string()))
    }

    Ok(())
}
