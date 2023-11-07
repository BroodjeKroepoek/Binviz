use std::{
    collections::BTreeMap,
    error::Error,
    fmt::Debug,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use comfy_table::{presets::ASCII_MARKDOWN, Table};
use image::{ImageBuffer, Luma};

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

/// Calculate the entropy from a given n-dimensional histogram.
pub fn calculate_entropy(histogram: &Histogram<u8>) -> f64 {
    let total: usize = histogram.values().sum();
    let entropy = histogram
        .iter()
        .map(|(_, freq)| {
            let relative_freq = (*freq as f64) / (total as f64);
            relative_freq.log2() * relative_freq
        })
        .sum::<f64>();
    -entropy
}

// TODO: Remove this?
/// Get the top `count` most frequent byte(s) from the given n-dimensional histogram.
pub fn get_most_frequent_bytes(histogram: &Histogram<u8>) -> Vec<(&Vec<u8>, &usize)> {
    let mut vector: Vec<(&Vec<u8>, &usize)> = histogram.into_iter().collect();
    vector.sort_by(|x, y| y.1.cmp(x.1));
    vector
}

pub fn display_entropies(entropy: f64, causal_entropy: f64) -> String {
    let rel_entropy = entropy / 8.0;
    let rel_causal_entropy = causal_entropy / 16.0;
    let mut table = Table::new();
    table.load_preset(ASCII_MARKDOWN);
    table.set_header(["Type", "Value", "Relative"]);
    table.add_row([
        "entropy",
        &format!("{:.5} (bits/byte)", entropy),
        &format!("{:.5}", rel_entropy),
    ]);
    table.add_row([
        "causal entropy",
        &format!("{:.5} (bits/dword)", causal_entropy),
        &format!("{:.5}", rel_causal_entropy),
    ]);
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
        let relative_freq = (*freq as f64) / (total as f64);
        table.add_row([
            format!("{}", i),
            format!("{}", byte[0]),
            format!("{:#x}", byte[0]),
            format!("{:?}", byte[0] as char),
            format!("{:.5}", relative_freq),
        ]);
    }
    table.to_string()
}

pub fn generate_image(dihistogram: &Histogram<u8>) -> ImageBuffer<Luma<u16>, Vec<u16>> {
    debug_assert!(dihistogram.into_iter().all(|x| x.0.len() == 2));
    let mut image = ImageBuffer::new(256, 256);
    let len = dihistogram.values().len();
    let total = (dihistogram.values().sum::<usize>() as f64) / (len as f64);
    for slice in dihistogram.keys() {
        if let Some(freq) = dihistogram.get(slice) {
            let brightness = (*freq as f64) / total * (u16::MAX as f64);
            let pixel = Luma([brightness as u16]);
            image.put_pixel(slice[0] as u32, slice[1] as u32, pixel);
        }
    }
    image
}

pub fn full_analysis(files: Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for file in &files {
        // Create a folder for each file to store the analysis results.
        let folder_name = file.file_stem().unwrap().to_str().unwrap();
        let output_folder = Path::new("output").join(folder_name);

        if !output_folder.exists() {
            fs::create_dir_all(&output_folder)?;
        }

        // Perform the Ent subcommand.
        let histogram = calculate_histogram(&file, 1);
        let dihistogram = calculate_histogram(&file, 2);

        let entropy = calculate_entropy(&histogram);
        let causal_entropy = calculate_entropy(&dihistogram);
        let entropy_output = display_entropies(entropy, causal_entropy);
        fs::write(output_folder.join("entropy.txt"), entropy_output)?;

        // Perform the Fre subcommand.
        let most_frequent_output = display_most_frequent(&histogram);
        fs::write(
            output_folder.join("most_frequent.txt"),
            most_frequent_output,
        )?;

        // Perform the Vis subcommand.
        let image = generate_image(&dihistogram);
        image.save(output_folder.join("image.png"))?;

        println!("[INFO] Analysis for '{}' is complete.", file.display());
    }

    Ok(())
}
