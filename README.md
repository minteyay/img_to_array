Converts a palette file and an image file into C arrays to use with Arduino or other embedded systems where defining image data in flash memory instead of reading it from external files is useful. Storing image data as indices referring to a palette saves RAM in situations where you might want to keep a buffer of an external display.

Usable palette files are images (for example, exported as .png in Aseprite) with all the colours used in the image. The palette file can be a single line or multidimensional. If the same colour is present multiple times in the palette, the first instance of it is used.

Usable image files are formats where decoding is supported by the Image crate (https://crates.io/crates/image). For now, PNG is the only format that's been tested, but others should work as well. All of the colours found in the image must be found in the palette file, otherwise an error is generated.

Please note: Currently this only converts 32-bit RGBA images (should work with 24-bit RGB as well) into a 256-colour palette of RGB565 (16-bit) values and an array of pixel indices referring to the palette. If you'd like to use it for different formats, feel free to open an issue and I'll look into it!

## Example
As an example, here's a heart (7x7 pixels) and its palette (4 colours, 4x1 pixels, black is used as a transparency), scaled up to 800% here for clarity.

![a pixelart heart](https://raw.githubusercontent.com/minteyay/img_to_array/master/doc/example_heart_big.png "a pixelart heart")

![a palette of black and shades of red](https://raw.githubusercontent.com/minteyay/img_to_array/master/doc/example_heart_big_palette.png "a palette of black and shades of red")

Passing these files to the converter produces the following file:
```C
#include <stdint.h>

const uint16_t palette[4] PROGMEM = {
        0x0000, 0xB192, 0xEA6E, 0xFE59, 
};

const uint8_t image_data[49] PROGMEM = {
      0,  2,  2,  0,  2,  2,  0,  2,  3,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,
      2,  2,  1,  2,  2,  2,  2,  2,  1,  0,  1,  2,  2,  2,  1,  0,  0,  0,  1,
      2,  1,  0,  0,  0,  0,  0,  1,  0,  0,  0,
};
```

## Usage
```
Usage: img_to_array PALETTE_PATH IMAGE_PATH [options]

Options:
    -o, --output NAME   set output file name (output.c by default)
    -h, --help          print this help message
```
