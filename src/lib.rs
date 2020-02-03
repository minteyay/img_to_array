extern crate getopts;

use std::fs;
use std::collections::{ HashSet, HashMap };
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

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
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
    opts.optopt("c", "colour", "set colour format ([RGB]565, RGB[888]) (565 by \
        default)",
        "FORMAT");
    opts.optopt("p", "palette", "set palette file", "FILE");
    opts.optopt("", "palsize", "set palette size in bits (8, 16, 32) (8 by \
        default)",
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

    // Construct the palette
    let palette = construct_palette(&config, &img)?;

    // Add the palette to the output
    write_palette(&mut output, &config, &palette);

    // If we have a separate palette file, check that the image doesn't have
    // any colours not found in the palette
    if config.palette_path.is_some() {
        check_against_palette(&img, &palette)?;
    }

    // Add the image data array definition to the output
    write_image_data(&mut output, &config, &img, &palette);

    // Write output to file
    if let Err(e) = fs::write(&config.output_path, output) {
        return Err(format!("Error writing output file: {}", e.to_string()))
    }

    Ok(())
}

fn list_colours(palette_img: &image::DynamicImage) -> (Vec<Rgb>, HashSet<Rgb>) {
    let mut colours: HashSet<Rgb> = HashSet::new();
    let mut palette: Vec<Rgb> = Vec::new();
    let palette_img = palette_img.to_bgra();

    for pixel in palette_img.enumerate_pixels() {
        let colour = Rgb::from(pixel.2);
        if colours.insert(colour) {
            palette.push(colour);
        }
    }
    (palette, colours)
}

fn construct_palette(config: &Config, img: &image::DynamicImage)
    -> Result<Vec<Rgb>, String> {
    match &config.palette_path {
        Some(path) => {
            let palette_img = match image::open(&path) {
                Ok(img) => img,
                Err(e) => return Err(format!("Error opening palette file \
                    \"{}\": {}", &path, e.to_string())),
            };
            Ok(list_colours(&palette_img).0)
        },
        None => {
            let palette = list_colours(&img).0;
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
            Ok(palette)
        },
    }
}

fn write_palette(output: &mut String, config: &Config, palette: &Vec<Rgb>) {
    output.push_str("\nconst ");
    match config.colour_format {
        ColourFormat::RGB565 => output.push_str("uint16_t"),
        _ => output.push_str("uint32_t"),
    }
    output.push_str(format!(" palette[{}] PROGMEM = {{\n",
        palette.len()).as_str());

    let mut line = String::from("    ");
    let mut to_add: String;
    for colour in palette {
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
}

fn check_against_palette(img: &image::DynamicImage, palette: &Vec<Rgb>)
    -> Result<(), String> {
    let img_colours = list_colours(&img).1;
    for colour in img_colours.iter() {
        if palette.iter().position( |c| c == colour ).is_none() {
            return Err(format!("Colour {:#08X} isn't present in the palette",
                colour.0))
        }
    }
    Ok(())
}

fn write_image_data(output: &mut String, config: &Config,
    img: &image::DynamicImage, palette: &Vec<Rgb>) {
    // Create a hashmap for the palette so we don't need to search the vector
    let mut palette_map: HashMap<Rgb, usize> = HashMap::new();
    for index in 0..palette.len() {
        palette_map.insert(palette[index], index);
    }

    let img = img.to_bgra();
    output.push_str(format!("\nconst uint{}_t image_data[{}] PROGMEM = {{\n",
        config.palette_size, img.dimensions().0 * img.dimensions().1).as_str());
    let mut line = String::from("    ");
    let mut to_add: String;
    for pixel in img.enumerate_pixels() {
        let palette_index = palette_map.get(&Rgb::from(pixel.2)).unwrap();

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
}
