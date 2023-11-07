use clap::Parser;
use clap_derive::{Parser, Subcommand};
use std::{collections::BTreeMap, fmt::Debug, fs::File, io::Read, path::PathBuf};

use comfy_table::{presets::ASCII_MARKDOWN, Table};
use image::{ImageBuffer, Luma};

use binviz::{calculate_entropy, calculate_histogram, get_most_frequent_bytes};

#[derive(Debug, Clone, Subcommand)]
enum CliCommand {
    /// Calculate the entropy of a given file in bits per byte.
    Ent {
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Get the top `count` most frequent bytes or all in sorted order if `None` of a given file.
    Fre {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long)]
        count: Option<u8>,
    },
    /// Visualize the given file as an image (DiGraph analysis).
    ///
    /// We scan pair of bytes from the file and treat that as x and y coordinates into the image.
    /// The pixels brightness will correspond with how many occurrences we have.
    Vis {
        #[arg(short, long)]
        file: PathBuf,
    },
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        CliCommand::Ent { file } => {
            let histogram = calculate_histogram(file);
            let entropy = calculate_entropy(&histogram);
            println!("{:.5} / 8.00000", entropy);
        }
        CliCommand::Fre { file, count } => {
            let histogram = calculate_histogram(file);
            let most_freq = get_most_frequent_bytes(&histogram, count);
            let total: usize = histogram.values().sum();
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            table.set_header(["Rank", "Byte", "Hex", "Text", "Frequency"]);
            for (i, (byte, freq)) in most_freq.into_iter().enumerate() {
                let relative_freq = (*freq as f64) / (total as f64);
                table.add_row([
                    format!("{}", i),
                    format!("{}", byte),
                    format!("{:#x}", byte),
                    format!("{:?}", *byte as char),
                    format!("{:.5}", relative_freq),
                ]);
            }
            println!("{}", table);
        }
        CliCommand::Vis { file } => {
            let mut two_pairs: BTreeMap<(u8, u8), usize> = BTreeMap::new();
            let mut handle = File::open(&file).expect(&format!("Couldn't open file: {:?}", file));
            let mut bytes = Vec::new();
            handle
                .read_to_end(&mut bytes)
                .expect(&format!("Couldn't `read_to_end` on: {:?}", handle));
            for slice in bytes.windows(2) {
                two_pairs
                    .entry((slice[0], slice[1]))
                    .and_modify(|x| *x += 1)
                    .or_insert(1);
            }
            let mut image = ImageBuffer::new(256, 256);
            let len = two_pairs.values().len();
            let total = (two_pairs.values().sum::<usize>() as f64) / (len as f64);
            for (x, y) in two_pairs.keys() {
                if let Some(freq) = two_pairs.get(&(*x, *y)) {
                    let brightness = (*freq as f64) / total * (u16::MAX as f64);
                    let pixel = Luma([brightness as u16]);
                    image.put_pixel(*x as u32, *y as u32, pixel);
                }
            }
            image.save("output.png").expect("Couldn't save image");
        }
    }
}
