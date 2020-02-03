extern crate getopts;

use std::fs;
use getopts::Options;

#[derive(Debug)]
pub struct Config {
    pub image_path: String,
    pub palette_path: Option<String>,
    pub output_path: String,
    pub colour_format: ColourFormat,
    pub palette_size: u8,
}

#[derive(Debug)]
pub enum ColourFormat { RGB565, RGB }

#[derive(Debug)]
struct Rgb565(u16);

impl From<&Rgb> for Rgb565 {
    fn from(rgb: &Rgb) -> Self {
        // First we shift to the right to truncate the values to the widths
        // we want (5 most significant bits from red, 6 from green, 5 from blue)
        // then shift them back to the left to their places in 565
        let r: u16 = (((rgb.0 & 0xFF0000) >> 16) as u16) >> 3 << 11;
        let g: u16 = (((rgb.0 & 0x00FF00) >> 8) as u16) >> 2 << 5;
        let b: u16 = (((rgb.0 & 0x0000FF)) as u16) >> 3;
        Rgb565(r | g | b)
    }
}

#[derive(Debug, PartialEq)]
struct Rgb(u32);

impl From<&image::Bgra<u8>> for Rgb {
    fn from(bgra: &image::Bgra<u8>) -> Self {
        let r: u32 = u32::from(bgra[2]) << 16;
        let g: u32 = u32::from(bgra[1]) << 8;
        let b: u32 = u32::from(bgra[0]);
        Rgb(r | g | b)
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} IMAGE_PATH [options]", program);
    print!("{}", opts.usage(&brief));
}

pub fn parse_config(args: Vec<String>) -> Result<Config, ()> {
    // Get program name
    let program = args[0].clone();

    // Setup getopts options and flags
    let mut opts = Options::new();
    opts.optopt("c", "colour", "set colour format ([RGB]565, RGB[888])",
        "FORMAT");
    opts.optopt("p", "palette", "set palette file", "FILE");
    opts.optopt("", "palsize", "set palette size in bits (8, 16, 32)",
        "SIZE");
    opts.optopt("o", "output", "set output file name (output.c by default)",
        "FILE");
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
    if matches.free.len() != 1 {
        eprintln!("No image file specified");
        print_usage(&program, opts);
        return Err(())
    }

    // Set the output path to one specified or the default one
    let output_path = match matches.opt_str("o") {
        Some(v) => v,
        None => String::from("output.c"),
    };

    // Set the colour format to one specified or the default one
    let colour_format = match matches.opt_str("c") {
        Some(v) => match v.as_str() {
            "RGB565" | "565" => ColourFormat::RGB565,
            "RGB" | "RGB888" | "888" => ColourFormat::RGB,
            _ => {
                eprintln!("Unknown colour format {}", v);
                print_usage(&program, opts);
                return Err(())
            }
        },
        None => ColourFormat::RGB565,
    };

    // Set the palette size to one specified or the default one
    let palette_size = match matches.opt_str("palsize") {
        Some(v) => match v.as_str() {
            "8" => 8,
            "16" => 16,
            "32" => 32,
            _ => {
                eprintln!("Unknown palette size {}", v);
                print_usage(&program, opts);
                return Err(())
            }
        },
        None => 8,
    };

    let palette_path = matches.opt_str("p");
    let image_path = matches.free[0].clone();

    Ok(Config {
        palette_path,
        image_path,
        output_path,
        colour_format,
        palette_size,
    })
}

pub fn convert(config: &Config) -> Result<(), String> {
    let mut output = String::from("#include <stdint.h>\n");

    // Read in the image to convert
    let img = match image::open(&config.image_path) {
        Ok(img) => img,
        Err(e) => return Err(format!("Error opening image file \"{}\": {}",
            &config.image_path, e.to_string())),
    };
    let img = img.to_bgra();

    // Construct the palette
    let mut palette: Vec<Rgb> = Vec::new();
    match &config.palette_path {
        Some(path) => {
            // Read in and store the palette if one was given
            let palette_img = match image::open(&path) {
                Ok(img) => img,
                Err(e) => return Err(format!("Error opening palette file \
                    \"{}\": {}", &path, e.to_string())),
            };
            let palette_img = palette_img.to_bgra();

            // Add the colours from the palette to the vector
            for pixel in palette_img.enumerate_pixels() {
                palette.push(Rgb::from(pixel.2));
            }
        },
        None => {
            // Make a palette from the image if one wasn't given
            for pixel in img.enumerate_pixels() {
                let colour = Rgb::from(pixel.2);

                // Add the pixel's colour to the palette if it's not there yet
                match palette.iter().position( |c| c == &colour ) {
                    Some(_) => (),
                    None => {
                        if palette.len() > match config.palette_size {
                            8 => 256,
                            16 => 65536,
                            32 => 4294967296,
                            _ => 0,
                        } {
                            return Err(format!("Image file has too many \
                                colours for palette size of {}",
                                config.palette_size))
                        }
                        palette.push(colour);
                    }
                }
            }
        },
    }

    // Add the palette to the output
    output.push_str("\nconst ");
    match config.colour_format {
        ColourFormat::RGB565 => output.push_str("uint16_t"),
        _ => output.push_str("uint32_t"),
    }
    output.push_str(format!(" palette[{}] PROGMEM = {{\n    ",
        palette.len()).as_str());
    let mut line = String::from("    ");
    let mut to_add: String;
    for colour in &palette {
        match config.colour_format {
            ColourFormat::RGB565 => to_add = format!("{:#06X}, ",
                Rgb565::from(colour).0),
            ColourFormat::RGB => to_add = format!("{:#08X}, ", colour.0),
        }
        // Check if we need to push the current value to the next line
        if line.len() + to_add.len() > 80 {
            output.push_str(format!("{}\n", line).as_str());
            line = String::from("    ");
        }
        line.push_str(to_add.as_str());
    }
    if !line.trim().is_empty() {
        output.push_str(line.as_str());
    }
    output.push_str("\n};\n");

    // Add the image data array definition to the output
    output.push_str(format!("const uint{}_t image_data[{}] PROGMEM = {{\n",
        config.palette_size, img.dimensions().0 * img.dimensions().1).as_str());
    line = String::from("    ");
    for pixel in img.enumerate_pixels() {
        let colour = Rgb::from(pixel.2);

        // Check that the colour is defined in the palette
        let palette_index = match palette.iter().position( |c| c == &colour) {
            Some(v) => v,
            None => match config.colour_format {
                ColourFormat::RGB565 => return Err(format!("Error creating \
                    colour index array: colour {:#06X} ({:#08X}) isn't present \
                    in the palette", Rgb565::from(&colour).0, colour.0)),
                ColourFormat::RGB => return Err(format!("Error \
                    creating colour index array: colour {:#08X} isn't \
                    present in the palette", colour.0)),
            },
        };

        to_add = format!("{},", palette_index);
        // Check if we need to push the current value to the next line
        if line.len() + to_add.len() > 80 {
            output.push_str(format!("{}\n", line).as_str());
            line = String::from("    ");
        }

        line.push_str(to_add.as_str());
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
