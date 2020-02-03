# img_to_array

Converts an image file into C array(s) to use with Arduino or other embedded systems where defining image data in flash memory instead of reading it from external files is useful. Storing image data as indices referring to a palette saves RAM in situations where you might want to keep a buffer of an external display.

Usable image files are formats where decoding is supported by the Image crate (https://crates.io/crates/image). For now, PNG is the only format that's been tested, but others should work as well. If a palette file is given, all of the colours found in the image must also be found in the palette or an error is generated.

Usable palette files are images (for example, exported as .png in Aseprite) with all the colours used in the image. If a palette file isn't given, a palette is automatically created from the colours in the image. The palette file can be a single line or multidimensional. If the same colour is present multiple times in the palette, the first instance of it is used.

The palette generated can be configured to either use RGB565 (uint16) or RGB888 (uint32). Palette indexing can also be disabled with `--nopalette`, resulting in colour values being written directly to the image data array. Transparency in the input files is ignored. The size of the palette (and as a result, the size of the data type used for image data indices) can be set to 8, 16, or 32 bits.

## Example
As an example, here's a heart (7x7 pixels) and its palette (4 colours, 4x1 pixels, black is used as a transparency), scaled up to 800% here for clarity.

![a pixelart heart](https://raw.githubusercontent.com/minteyay/img_to_array/master/doc/example_heart_big.png "a pixelart heart")

![a palette of black and shades of red](https://raw.githubusercontent.com/minteyay/img_to_array/master/doc/example_heart_palette_big.png "a palette of black and shades of red")

Passing these files to the converter with `img_to_array example_heart.png -p example_heart_palette.png` produces the following file:
```C
#include <stdint.h>

const uint16_t palette[4] PROGMEM = {
        0x0000, 0xB192, 0xEA6E, 0xFE59, 
};

const uint8_t image_data[49] PROGMEM = {
    0,2,2,0,2,2,0,2,3,2,2,2,2,2,2,2,2,2,2,2,2,1,2,2,2,2,2,1,0,1,2,2,2,1,0,0,0,1,
    2,1,0,0,0,0,0,1,0,0,0,
};
```

Passing only the image file with `img_to_array example_heart.png --nopalette` to write the colour values directly into the array without using a palette produces the following file:
```C
#include <stdint.h>

const uint16_t image_data[49] PROGMEM = {
    0x0000,0xEA6E,0xEA6E,0x0000,0xEA6E,0xEA6E,0x0000,0xEA6E,0xFE59,0xEA6E,
    0xEA6E,0xEA6E,0xEA6E,0xEA6E,0xEA6E,0xEA6E,0xEA6E,0xEA6E,0xEA6E,0xEA6E,
    0xEA6E,0xB192,0xEA6E,0xEA6E,0xEA6E,0xEA6E,0xEA6E,0xB192,0x0000,0xB192,
    0xEA6E,0xEA6E,0xEA6E,0xB192,0x0000,0x0000,0x0000,0xB192,0xEA6E,0xB192,
    0x0000,0x0000,0x0000,0x0000,0x0000,0xB192,0x0000,0x0000,0x0000,
};
```

## Usage
```
Usage: img_to_array IMAGE_PATH [options]

Options:
    -c, --colour FORMAT set colour format ([RGB]565, RGB[888]) (565 by
                        default)
    -p, --palette FILE  set palette file
        --palsize SIZE  set palette size in bits (8, 16, 32) (8 by default)
        --nopalette     don't use a palette, just write colour values directly
    -o, --output FILE   set output file name (output.c by default)
    -h, --help          print this help message
```

Written by sam / minteyay! (@mintey@chitter.xyz)
