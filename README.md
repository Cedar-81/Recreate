# Recreate: Image Collage Generator

`Recreate` is a Rust-based CLI tool for generating collages by blending images with dominant colors from specific regions of a reference image. The tool supports parallel processing for faster image handling.

## Features
- **Create Collages:** Recreate a reference image using smaller images from a specified directory.
- **Grid Control:** Configure the number of rows and columns in the collage grid.
- **Blending:** Control how much the images blend with the dominant color of their respective grid region.
- **Multithreading:** Utilizes multi-threading to improve performance when processing large sets of images.

## Installation

1. Ensure you have Rust installed on your machine. If not, download and install it from [here](https://www.rust-lang.org/tools/install).
2. Clone the repository:
    ```bash
    git clone https://github.com/yourusername/recreate.git
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
- `-d, --dir <image-directory>`: The relative path to the directory containing the images used in the collage.
- `-p, --ref <reference-image>`: The relative path to the reference image to be recreated.

### Optional Arguments:
- `-c, --cols <grid-cols>`: Number of columns in the collage grid (default is `70`).
- `-r, --rows <grid-rows>`: Number of rows in the collage grid (default is `70`).
- `-a, --alpha <blending-factor>`: Controls how much the images blend with the dominant color (default is `0.7`).

### Example:
```bash
./target/release/recreate -d ./images -p ./reference.jpg -c 100 -r 100 -a 0.8
```

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
