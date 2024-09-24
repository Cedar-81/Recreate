# Recreate: Image Collage Generator

`Recreate` is a Rust-based CLI tool for generating collages by blending images with dominant colors from specific regions of a reference image. The tool supports parallel processing for faster image handling.

## Features

- **Create Collages:** Recreate a reference image using smaller images from a specified directory.
- **Grid Control:** Configure the number of rows and columns in the collage grid.
- **Blending:** Control how much the images blend with the dominant color of their respective grid region.
- **Multithreading:** Utilizes multi-threading to improve performance when processing large sets of images.

## Project Inspiration

I saw this fan art project that was created by the offical Olivia Rodrigo fan account- one of my favorite artists, and I thought it looked cool [twitter post here](https://x.com/LiviesHQ/status/1833234088523927813). I decided to create a Rust tool that lets you do something similar. All you need to do is specify an image directory and some command-line options, and you're good to go! <br/>
Here is the fan art/GUTS album cover collage
    <p align="center">
        <img src="https://pbs.twimg.com/media/GXD0rxxWgAA2iHf?format=jpg&name=large" alt="image from the twitter post" width="400"/>
    </p>

## Installation

1. Ensure you have Rust installed on your machine. If not, download and install it from [here](https://www.rust-lang.org/tools/install).
2. Clone the repository:
    
    ```bash
    git clone <https://github.com/yourusername/recreate.git>
    ```
    
3. Navigate to the directory:
    
    ```bash
    cd recreate
    ```
    
4. Build the project:
    
    ```bash
    cargo build --release
    ```
    

## Usage

Run the program using the following command:

```bash
./target/release/recreate -d <image-directory> -p <reference-image> [OPTIONS]

```

### Required Arguments:

- `d, --dir <image-directory>`: The relative path to the directory containing the images used in the collage.
- `p, --ref <reference-image>`: The relative path to the reference image to be recreated.

### Optional Arguments:

- **`c, --cols <COLS>`**
    
    Number of columns in the collage grid.
    
    If not provided, the value defaults to 70. This value is adjusted to the nearest multiple of the reference image's width, if necessary.
    
- **`r, --rows <ROWS>`**
    
    Number of rows in the collage grid.
    
    If not provided, the value defaults to 70. This value is adjusted to the nearest multiple of the reference image's height, if necessary.
    
- **`a, --alpha <ALPHA>`**
    
    Specifies the blending factor between the reference image's dominant color and the smaller images.
    
    Defaults to 0.7 (70% blend).
    
- **`v, --verbose`**
    
    Enables verbose output to print additional information about the process.
    
    Defaults to `true`.
    
- **`c, --resize`**
    
    Resizes the reference image to a square layout using its width. Prevents the adjustment of specified grid columns and rows.
    
    Defaults to `true`.
    
- **`s, --scale <SCALE>`**
    
    Scales the output image by multiplying its dimensions (width and height) by the specified value.
    
    Defaults to 0.0, meaning no scaling.
    

### Example:

Here’s an example showing how to use `Recreate`:

```bash
./target/release/recreate -d ./guts -p ./guts/g_ref4.webp -c 200 -r 200 -a 0.7 -s 2.0
```

### Arguments:

- `d ./guts`: Specifies the directory (`./guts`) containing the smaller images used in the collage.
- `p ./guts/g_ref4.webp`: Specifies the reference image (`g_ref4.webp`) that the collage will recreate.
- `c 200`: Sets the number of columns in the collage grid to 200. This may be adjusted based on the reference image.
- `r 200`: Sets the number of rows in the collage grid to 200.
- `a 0.7`: Sets the blending ratio, meaning 70% of the final color will be the dominant color of each grid region.
- `s 2.0`: Scales the final output image by a factor of 2.0, doubling its resolution.

### Initial Image Details:

- **Reference Image Resolution**: 770x746
- **Time Taken**: ~2mins `this can vary from pc to pc due to threading`
<p align="center">
    <img src="https://github.com/Cedar-81/Recreate/blob/main/example_images/g_ref3.webp" alt="guts olivia rodrigo alt album cover" width="400"/>
    <img src="https://github.com/Cedar-81/Recreate/blob/main/example_images/output.png" alt="guts olivia rodrigo alt album cover collage" width="400"/>
</p>
  

## How It Works

1. **Reading Images:** The tool reads all images in the specified directory (except the reference image).
2. **Collage Grid:** The tool divides the reference image into a grid based on the specified number of rows and columns.
3. **Blending:** Each grid section is filled with a resized image from the directory. The color of each image is blended with the dominant color of the corresponding grid section using the specified alpha value.
4. **Multithreading:** Image reading and processing are done in parallel using 20 threads for efficient performance.

## Output

The final collage is saved as `output.png` in the `guts` folder.

## Dependencies

- [Image](https://crates.io/crates/image) - Image processing library
- [Rayon](https://crates.io/crates/rayon) - For parallel processing
- [Anyhow](https://crates.io/crates/anyhow) - Error handling
- [Clap](https://crates.io/crates/clap) - Command-line argument parsing

## Contributing

Contributions are welcome! Please fork this repository and submit a pull request.

## License

This project is licensed under the MIT License.
