use anyhow::{anyhow, Context, Result};
use clap::Parser;
use image::{imageops::FilterType, open, DynamicImage, GenericImageView, ImageBuffer, Rgba};
use kmeans_colors::{get_kmeans, Kmeans, Sort};
use palette::cast::from_component_slice;
use palette::{FromColor, IntoColor, Lab, Srgb, Xyz};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::{
    fs,
    ops::{Add, Mul},
    sync::{Arc, RwLock},
    thread,
};

#[derive(Debug, Clone, Copy)]
struct RgbaWrapper(Rgba<u8>);

// impl RgbaWrapper {
//     /// Creates a new `RgbaWrapper` from an `Rgba<u8>`.
//     fn _new(rgba: Rgba<u8>) -> Self {
//         RgbaWrapper(rgba)
//     }
// }

// Implement multiplication by f32
impl Mul<f32> for RgbaWrapper {
    type Output = RgbaWrapper;

    fn mul(self, scalar: f32) -> Self::Output {
        let Rgba([r, g, b, a]) = self.0;

        // Scale each channel and clamp between 0 and 255
        let scaled = [
            (r as f32 * scalar).min(255.0).max(0.0) as u8,
            (g as f32 * scalar).min(255.0).max(0.0) as u8,
            (b as f32 * scalar).min(255.0).max(0.0) as u8,
            a, // Keep alpha unchanged
        ];

        RgbaWrapper(Rgba(scaled))
    }
}

// Implement addition of two RgbaWrapper instances
impl Add for RgbaWrapper {
    type Output = RgbaWrapper;

    fn add(self, other: RgbaWrapper) -> Self::Output {
        let Rgba([r1, g1, b1, a1]) = self.0;
        let Rgba([r2, g2, b2, a2]) = other.0;

        // Sum the channels and clamp between 0 and 255
        let summed = [
            (r1 as u16 + r2 as u16).min(255) as u8,
            (g1 as u16 + g2 as u16).min(255) as u8,
            (b1 as u16 + b2 as u16).min(255) as u8,
            (a1 as u16 + a2 as u16).min(255) as u8,
        ];

        RgbaWrapper(Rgba(summed))
    }
}

#[derive(Parser, Debug)]
#[command(name = "Recreate", version="1.0", about, long_about = None)]
struct Args {
    /// Relative path to directory containing images for collage
    #[arg(short, long)]
    dir: String,

    /// Relative path to the image to be recreated
    #[arg(short = 'p', long)]
    r#ref: String,

    /// Number of columns in the collage grid
    /// If not passed this value is set to 70 by default
    /// Note: If need be this is usually adjusted to the nearest multiple of the reference image's width that is greater than the specified value.
    #[arg(short, long, default_value_t = 70)]
    cols: u32,

    /// Number of columns in the collage grid
    /// If not passed this value is set to 70 by default
    /// Note: If need be this is usually adjusted to the nearest multiple of the reference image's height that is greater than the specified value.
    #[arg(short, long, default_value_t = 70)]
    rows: u32,

    /// This inidates how much the images are blended to look more like the dominant color of its placement position
    /// If not passed this value is set to 0.7 by default
    #[arg(short, long, default_value_t = 0.7)]
    alpha: f32,
}

#[derive(Debug, Default)]
struct Recreate {
    img_list: Arc<RwLock<Vec<DynamicImage>>>,
}

impl Recreate {
    fn new() -> Self {
        Self {
            img_list: Recreate::default().img_list,
        }
    }

    fn read_dir_to_vec(&mut self, dir_path: &str, ref_img: &str) -> Result<()> {
        println!("pulling images...");
        const NTHREADS: u32 = 20;
        let mut children = vec![];

        // Clone the Arc<Mutex<>> to move into threads
        let img_list = Arc::clone(&self.img_list);

        let files = fs::read_dir(dir_path).with_context(|| {
            format!(
                "Couldn't read directory in specified path: {}, do well to check the path again.",
                dir_path
            )
        })?;

        // Collect files before threads (avoid borrowing issues)
        let file_paths: Vec<_> = files
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();

        // Split the file paths into chunks for each thread
        let chunk_size = (file_paths.len() + NTHREADS as usize - 1) / NTHREADS as usize;
        let file_chunks: Vec<_> = file_paths.chunks(chunk_size).collect();

        // Spawn threads
        for chunk in file_chunks {
            let img_list = Arc::clone(&img_list); // Clone for thread safety
            let chunk = chunk.to_vec(); // Clone file chunk for this thread
            let ref_img_cp = ref_img.to_owned();

            children.push(thread::spawn(move || -> Result<()> {
                let mut local_vec = Vec::new(); // Local vec to batch insertions

                for file_path in chunk {
                    let file_name = file_path.file_name().unwrap();
                    let file_path_str = file_name.to_str().unwrap();

                    if file_path_str == ref_img_cp.as_str() {
                        continue;
                    }

                    let img = open(file_path.to_str().unwrap()).with_context(|| {
                        format!("Couldn't open image in specified path: {}", file_path_str)
                    })?;

                    local_vec.push(img);
                }

                // Batch insert results from local_map into the shared dom_map
                let mut list = img_list.write().unwrap();
                list.extend(local_vec);

                Ok(())
            }));
        }

        // Join all threads and handle potential errors
        for child in children {
            if let Err(e) = child.join().unwrap() {
                eprintln!("Thread error: {:?}", e); // Handle thread errors
            }
        }

        Ok(())
    }

    fn collage(&mut self, path: &str, grid_rows: u32, grid_cols: u32, alpha: f32) -> Result<()> {
        println!("collaging...");
        let mut img = open(path)
            .with_context(|| format!("Couldn't open image in specified path: {}", path))?;

        let (img_width, img_height) = img.dimensions();
        println!(
            "ref_img_width: {}, ref_img_height: {}",
            img_width, img_height
        );
        let grid_cols = next_divisor(img_width, grid_cols)?;
        let grid_rows = next_divisor(img_height, grid_rows)?;
        println!("grid_cols: {}, grid_rows: {}", grid_cols, grid_rows);

        let image_grid = divide_image_into_grid(&mut img, grid_cols, grid_rows);

        // Create a shared buffer for the reconstructed image using Mutex for safe access
        let reconstructed_img_buffer = Arc::new(RwLock::new(
            ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(img_width, img_height),
        ));

        // Parallel processing of image grid portions
        image_grid
            .par_iter()
            .enumerate()
            .for_each(|(idx, portion)| {
                // Create a new RNG for each thread to avoid non-Sync error
                let mut rng = StdRng::from_entropy();

                let (p_width, p_height) = portion.dimensions();
                let img_list = self.img_list.read().unwrap().clone();
                let random_number = rng.gen_range(0..img_list.len());

                // Resize the image to match the current portion dimensions
                let resized_img =
                    img_list[random_number].resize_exact(p_width, p_height, FilterType::Lanczos3);

                // dominant color in portion
                let portion_bytes = portion.as_rgb8().unwrap().clone().into_raw();
                let dom_color = lab_to_rgba_u8(calc_dominant_color(portion_bytes));

                let grid_x = idx as u32 % grid_cols;
                let grid_y = idx as u32 / grid_cols;
                let x_start = grid_x * p_width;
                let y_start = grid_y * p_height;

                for y in 0..p_height {
                    for x in 0..p_width {
                        if (x_start + x) < img_width && (y_start + y) < img_height {
                            let pixel = resized_img.get_pixel(x, y);
                            //blend pixel color with dominant color using LERP
                            let p_final =
                                RgbaWrapper(pixel) * (1.0 - alpha) + RgbaWrapper(dom_color) * alpha;
                            reconstructed_img_buffer.write().unwrap().put_pixel(
                                x_start + x,
                                y_start + y,
                                p_final.0,
                            );
                        }
                    }
                }
            });

        // Save the output image
        reconstructed_img_buffer
            .read()
            .unwrap()
            .save("./guts/output.png")
            .with_context(|| format!("Couldn't save image"))?;

        Ok(())
    }
}

fn main() -> Result<()> {
    // println!("image vec: {:?}", a);

    let args = Args::parse();
    let split_ref_path: Vec<&str> = args.r#ref.split("/").collect();
    println!(
        "Args: {:?}, {:?}",
        args,
        split_ref_path[split_ref_path.len() - 1]
    );

    let mut recreate = Recreate::new();
    let _ = recreate.read_dir_to_vec(&args.dir, split_ref_path[split_ref_path.len() - 1])?;
    let _ = recreate.collage(&args.r#ref, args.rows, args.cols, args.alpha)?;

    Ok(())
}

fn divide_image_into_grid(
    image: &mut DynamicImage,
    grid_width: u32,
    grid_height: u32,
) -> Vec<DynamicImage> {
    let (img_width, img_height) = image.dimensions();

    // Calculate the "ideal" width and height of each grid cell
    //basically if we want to have m rows and n cols we need to divide the img_width and img_height
    //by the number of cols and number of rows
    let cell_width = img_width / grid_width;
    let cell_height = img_height / grid_height;

    // println!("cell_width: {}, cell_height: {}", cell_width, cell_height);

    let mut grid_cells = Vec::new();

    for y in 0..grid_height {
        for x in 0..grid_width {
            // Calculate start and end coordinates for this cell
            let x_start = x * cell_width;
            let y_start = y * cell_height;

            // // For the last column, ensure the width matches the remaining image width
            // let x_end = if x == grid_width - 1 {
            //     img_width
            // } else {
            //     (x + 1) * cell_width
            // };

            // // For the last row, ensure the height matches the remaining image height
            // let y_end = if y == grid_height - 1 {
            //     img_height
            // } else {
            //     (y + 1) * cell_height
            // };

            // Create the sub-image (portion) for this grid cell
            let cell_image = image.crop(x_start, y_start, cell_width, cell_height);
            grid_cells.push(cell_image);
        }
    }

    // println!(
    //     "grid len: {}, grid dimensions: {:?}",
    //     grid_cells.len(),
    //     grid_cells[0].dimensions()
    // );
    grid_cells
}

fn next_divisor(n: u32, start: u32) -> Result<u32> {
    if start > n {
        return Err(anyhow!("Grid value should be less that {}", n));
    }

    if n % start == 0 {
        return Ok(start);
    }

    for i in (start + 1)..=n {
        if n % i == 0 {
            return Ok(i); // Return the next divisor
        }
    }

    Ok(start)
}

fn lab_to_rgba_u8(lab: Lab) -> Rgba<u8> {
    // Convert Lab to XYZ
    let xyz: Xyz = Xyz::from_color(lab);

    // Convert XYZ to Srgb (RGB)
    let rgb: Srgb = Srgb::from_color(xyz);

    // Clamp RGB values and convert to u8
    let r = (rgb.red * 255.0).clamp(0.0, 255.0) as u8;
    let g = (rgb.green * 255.0).clamp(0.0, 255.0) as u8;
    let b = (rgb.blue * 255.0).clamp(0.0, 255.0) as u8;

    // Return as RGBA (with full opacity)
    Rgba([r, g, b, 255])
}

fn calc_dominant_color(img_vec: Vec<u8>) -> Lab {
    // Convert RGB [u8] buffer to Lab for k-means
    let lab: Vec<Lab> = from_component_slice::<Srgb<u8>>(&img_vec)
        .iter()
        .map(|x| x.into_format().into_color())
        .collect();

    // Iterate over the runs, keep the best results
    let mut result = Kmeans::new();
    for i in 0..3 {
        let run_result = get_kmeans(8, 20, 5.0, false, &lab, 30 + i as u64);
        if run_result.score < result.score {
            result = run_result;
        }
    }

    // Using the results, process the centroid data
    let res = Lab::sort_indexed_colors(&result.centroids, &result.indices);

    // We can find the dominant color directly
    let dominant_color = Lab::get_dominant_color(&res);

    dominant_color.unwrap()
}
