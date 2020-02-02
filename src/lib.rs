use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub image_path: String,
    pub palette_path: String,
    pub output_path: String,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        // TODO: proper getopts parsing with error handling
        args.next();

        let image_path = match args.next() {
            Some(arg) => arg,
            None => return Err("No image file path given"),
        };
        let palette_path = match args.next() {
            Some(arg) => arg,
            None => return Err("No palette file path given"),
        };
        let output_path = match args.next() {
            Some(arg) => arg,
            None => return Err("No output file path given"),
        };

        Ok(Config { image_path, palette_path, output_path })
    }
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

pub fn convert(config: Config) {
    let mut output = String::from("#include <stdint.h>\n");

    // Read in and store the palette
    // TODO: error handling (mostly incorrect path)
    let palette_img = image::open(config.palette_path).unwrap().to_bgra();
    let mut palette: HashMap<Rgb565, usize> = HashMap::new();

    for pixel in palette_img.enumerate_pixels() {
        let colour = Rgb565::from(pixel.2);
        if ! palette.contains_key(&colour) {
            palette.insert(colour, palette.len());
        }
    }

    // Add the palette to the output
    output.push_str(format!("\nconst uint16_t palette[{}] PROGMEM = {{\n    ",
        palette.len()).as_str());
    let mut line = String::from("    ");
    for colour in &palette {
        if line.len() + 7 > 80 {
            output.push_str(format!("{}\n", line).as_str());
            line = String::from("    ");
        }
        line.push_str(format!("{:#06X}, ", (colour.0).0).as_str());
    }
    if !line.trim().is_empty() {
        output.push_str(line.as_str());
    }
    output.push_str("\n};\n\n");

    // Read in the image to convert and add its pixels to the output
    // TODO: error handling (mostly incorrect path)
    let img = image::open(config.image_path).unwrap().to_bgra();
    output.push_str(format!("const uint8_t image_data[{}] PROGMEM = {{\n",
        img.dimensions().0 * img.dimensions().1).as_str());
    let mut line = String::from("    ");
    for pixel in img.enumerate_pixels() {
        let colour = Rgb565::from(pixel.2);
        if line.len() + 4 > 80 {
            output.push_str(format!("{}\n", line).as_str());
            line = String::from("    ");
        }
        // TODO: error handling and reporting
        // (image may have colours not present in the palette)
        line.push_str(format!("{:3},", palette.get(&colour).unwrap()).as_str());
    }
    if !line.trim().is_empty() {
        output.push_str(format!("{}\n", line).as_str());
    }
    output.push_str("};\n");

    // Write output to file
    // TODO: error handling
    fs::write(config.output_path, output)
        .expect("Couldn't open file for writing");
}
