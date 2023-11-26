use std::{
    collections::BTreeMap,
    fmt::Debug,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use comfy_table::{presets::ASCII_MARKDOWN, Table};
use image::{ImageBuffer, Luma, Rgb};
use log::info;

type Histogram<T> = BTreeMap<Vec<T>, usize>;

/// Calculate the n-dimensional histogram of (consecutive) bytes of a given file.
pub fn calculate_histogram<P>(file: P, dimension: usize) -> Histogram<u8>
where
    P: AsRef<Path> + Debug,
{
    let mut histogram = BTreeMap::new();
    let mut handle = File::open(&file).expect(&format!("Couldn't open file: {:?}", file));
    let mut buf = Vec::new();
    handle
        .read_to_end(&mut buf)
        .expect(&format!("Couldn't `read_to_end` on: {:?}", handle));
    for byte in buf.windows(dimension) {
        histogram
            .entry(byte.to_vec())
            .and_modify(|x| *x += 1)
            .or_insert(1);
    }
    histogram
}

#[inline(always)]
pub fn calculate_entropy(probability: f64) -> f64 {
    probability.log2() * probability
}

/// Calculate the entropy from a given n-dimensional histogram.
pub fn calculate_entropy_histogram(histogram: &Histogram<u8>) -> f64 {
    let total: usize = histogram.values().sum();
    let entropy = histogram
        .iter()
        .map(|(_, freq)| {
            let probability = (*freq as f64) / (total as f64);
            calculate_entropy(probability)
        })
        .sum::<f64>();
    -entropy
}

pub fn get_most_frequent_bytes(histogram: &Histogram<u8>) -> Vec<(&Vec<u8>, &usize)> {
    let mut vector: Vec<(&Vec<u8>, &usize)> = histogram.into_iter().collect();
    vector.sort_by(|x, y| y.1.cmp(x.1));
    vector
}

pub fn display_entropies<P>(file: P, count: usize) -> String
where
    P: AsRef<Path> + Debug,
{
    let mut table = Table::new();
    table.load_preset(ASCII_MARKDOWN);
    table.set_header(["Dimension", "Entropy", "Relative Entropy"]);
    for i in 1..=count {
        let histogram = calculate_histogram(&file, i);
        let entropy = calculate_entropy_histogram(&histogram);
        let rel_entropy = entropy / (8.0f64 * (i as f64));
        table.add_row([
            format!("{}", i),
            format!("{:.5} (bits per {} byte(s))", entropy, i),
            format!("{:.5}", rel_entropy),
        ]);
    }
    table.to_string()
}

pub fn display_most_frequent(histogram: &Histogram<u8>) -> String {
    debug_assert!(histogram.into_iter().all(|x| x.0.len() == 1));
    let total: usize = histogram.values().sum();
    let most_freq = get_most_frequent_bytes(histogram);
    let mut table = Table::new();
    table.load_preset(ASCII_MARKDOWN);
    table.set_header(["Rank", "Byte", "Hex", "Text", "Relative Frequency"]);
    for (i, (byte, freq)) in most_freq.into_iter().enumerate() {
        let probability = (*freq as f64) / (total as f64);
        table.add_row([
            format!("{}", i),
            format!("{}", byte[0]),
            format!("{:#x}", byte[0]),
            format!("{:?}", byte[0] as char),
            format!("{:.5}", probability),
        ]);
    }
    table.to_string()
}

pub fn generate_image(
    dihistogram: &Histogram<u8>,
) -> (ImageBuffer<Luma<u16>, Vec<u16>>, usize, f64) {
    debug_assert!(dihistogram.into_iter().all(|x| x.0.len() == 2));
    let mut image = ImageBuffer::new(256, 256);
    let len = dihistogram.values().len();
    let total: usize = dihistogram.values().sum();
    let avg_total = (total as f64) / (len as f64);
    for slice in dihistogram.keys() {
        if let Some(freq) = dihistogram.get(slice) {
            let brightness = (*freq as f64) / avg_total * (u16::MAX as f64);
            let pixel = Luma([brightness as u16]);
            image.put_pixel(slice[0] as u32, slice[1] as u32, pixel);
        }
    }
    (image, total, avg_total)
}

// [u8; 3] -> usize
// slice[0] x coordinate
// slice[1] y coordinate
// slice[2] right now: red component
// value right now: blue component
// A pixel just existing adds full green component, for easier distinction vs not existent pixels.
pub fn generate_color_image(
    trihistogram: &Histogram<u8>,
) -> (ImageBuffer<Rgb<u16>, Vec<u16>>, usize, f64) {
    debug_assert!(trihistogram.into_iter().all(|x| x.0.len() == 3));
    let mut image = ImageBuffer::new(256, 256);
    let len = trihistogram.values().len();
    let total: usize = trihistogram.values().sum();
    let avg_total = (total as f64) / (len as f64);
    for slice in trihistogram.keys() {
        if let Some(freq) = trihistogram.get(slice) {
            // dividing by avg_total makes it so we actually see something, by the pixel overflows if *freq* is more the the average value.
            // by len takes it into account properly?????
            let brightness_2 = (*freq as f64) * (u16::MAX as f64) / (avg_total as f64);
            let brightness_1 = (slice[2] as f64) * (u16::MAX as f64) / (u8::MAX as f64);
            let pixel = Rgb([brightness_1 as u16, 0, brightness_2 as u16]);
            image.put_pixel(slice[0] as u32, slice[1] as u32, pixel);
        }
    }
    (image, total, avg_total)
}

/// Perform a full analysis on all the files provided.
pub fn full_analysis(files: Vec<PathBuf>) {
    for file in &files {
        // Create a folder for each file to store the analysis results.
        let folder_name = file
            .file_stem()
            .expect("The file has no filename")
            .to_str()
            .expect("The path is not valid Unicode");
        let output_folder = Path::new("output").join(folder_name);

        if !output_folder.exists() {
            fs::create_dir_all(&output_folder)
                .expect(&format!("Couldn't `create_dir_all` on {:?}", output_folder));
        }

        // Perform the Ent subcommand.
        let entropy_output = display_entropies(&file, 3);
        fs::write(output_folder.join("entropy.txt"), entropy_output)
            .expect("Couldn't write into 'entropy.txt'");

        // Perform the Fre subcommand.
        let histogram = calculate_histogram(&file, 1);
        let most_frequent_output = display_most_frequent(&histogram);
        fs::write(
            output_folder.join("most_frequent.txt"),
            most_frequent_output,
        )
        .expect("Couldn't write into `most_frequent.txt`");

        // Perform the Vis subcommand.
        let dihistogram = calculate_histogram(&file, 2);
        let (image, total, avg_total) = generate_image(&dihistogram);
        image
            .save(output_folder.join("image.png"))
            .expect("Couldn't save image into `image.png`");
        info!("`{}` byte pairs in the visualization.", total);
        info!(
            "full brightness means `{}` byte pairs at that location.",
            avg_total
        );
        info!("Analysis for '{}' is complete.", file.display());
    }
}
