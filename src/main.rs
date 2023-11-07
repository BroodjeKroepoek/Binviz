use clap::Parser;
use clap_derive::{Parser, Subcommand};
use std::{fmt::Debug, path::PathBuf};

use binviz::{
    calculate_entropy, calculate_histogram, display_entropies, display_most_frequent,
    full_analysis, generate_image,
};

#[derive(Debug, Clone, Subcommand)]
enum CliCommand {
    /// Calculate the entropy and causal entropy of a given file in bits per byte and bits per dword respectively.
    Ent {
        #[arg(short, long)]
        file: PathBuf,
    },
    // TODO: Add the most frequent conditional byte using a di-histogram.
    /// Get the bytes in sorted order according to their frequency of a given file.
    Fre {
        #[arg(short, long)]
        file: PathBuf,
    },
    // TODO: Train a NN on this, to classify file formats?
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
    /// Perform a full analysis, by performing all other commands on every file
    /// and collecting the output into a folders corresponding to each file.
    Full {
        #[arg(short, long)]
        files: Vec<PathBuf>,
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
            let histogram = calculate_histogram(&file, 1);
            let dihistogram = calculate_histogram(&file, 2);
            let entropy = calculate_entropy(&histogram);
            let causal_entropy = calculate_entropy(&dihistogram);
            println!("{}", display_entropies(entropy, causal_entropy));
        }
        CliCommand::Fre { file } => {
            let histogram = calculate_histogram(&file, 1);
            println!("{}", display_most_frequent(&histogram));
        }
        CliCommand::Vis { file } => {
            let dihistogram = calculate_histogram(&file, 2);
            println!("[INFO] generating image...");
            let image = generate_image(&dihistogram);
            println!("[INFO] finished");
            image.save("output.png").expect("Couldn't save image");
            println!("[INFO] image saved to '.\\output.png'.");
        }
        CliCommand::Full { files } => {
            if let Err(err) = full_analysis(files) {
                eprintln!("Error during full analysis: {}", err);
            }
        }
    }
}
