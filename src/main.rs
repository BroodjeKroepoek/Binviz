use clap::Parser;
use clap_derive::{Parser, Subcommand};
use std::{fmt::Debug, path::PathBuf};

use comfy_table::{presets::ASCII_MARKDOWN, Table};
use image::{ImageBuffer, Luma};

use binviz::{
    calculate_causal_entropy, calculate_dihistogram, calculate_entropy, calculate_histogram,
    get_most_frequent_bytes,
};

#[derive(Debug, Clone, Subcommand)]
enum CliCommand {
    /// Calculate the entropy and causal entropy of a given file in bits per byte and bits per dword respectively.
    Ent {
        #[arg(short, long)]
        file: PathBuf,
    },
    // TODO: Add the most frequent conditional byte using a di-histogram.
    /// Get the top `count` most frequent bytes, or all if `None`, in sorted order of a given file.
    Fre {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long)]
        count: Option<u8>,
    },
    // TODO: Train a NN on this?
    /// Visualize the given file as an image (digraph analysis).
    ///
    /// We scan pair of bytes from the file and treat that as x and y coordinates into the image.
    /// The pixels brightness will correspond with how many occurrences we have.
    ///
    /// This can show conditional relationships within a binary file.
    /// Distinct file formats will produce distinct recognizable patterns in the image.
    Vis {
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Perform a full analysis, by performing all other commands on it and collecting the output into a folder.
    Full {
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
            let histogram = calculate_histogram(&file);
            let dihistogram = calculate_dihistogram(&file);
            let entropy = calculate_entropy(&histogram);
            let rel_entropy = entropy / 8.0;
            let causal_entropy = calculate_causal_entropy(&dihistogram);
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
            println!("{}", table);
        }
        CliCommand::Fre { file, count } => {
            let histogram = calculate_histogram(&file);
            let most_freq = get_most_frequent_bytes(&histogram, count);
            let total: usize = histogram.values().sum();
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            table.set_header(["Rank", "Byte", "Hex", "Text", "Relative Frequency"]);
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
            let dihistogram = calculate_dihistogram(&file);
            let mut image = ImageBuffer::new(256, 256);
            let len = dihistogram.values().len();
            let total = (dihistogram.values().sum::<usize>() as f64) / (len as f64);
            for (x, y) in dihistogram.keys() {
                if let Some(freq) = dihistogram.get(&(*x, *y)) {
                    let brightness = (*freq as f64) / total * (u16::MAX as f64);
                    let pixel = Luma([brightness as u16]);
                    image.put_pixel(*x as u32, *y as u32, pixel);
                }
            }
            image.save("output.png").expect("Couldn't save image");
        }
        CliCommand::Full { file: _ } => todo!(),
    }
}
