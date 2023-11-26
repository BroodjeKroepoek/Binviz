use clap::Parser;
use clap_derive::{Parser, Subcommand};
use comfy_table::{presets::ASCII_MARKDOWN, Table};
use env_logger::Env;

use log::info;
use std::{fmt::Debug, path::PathBuf, time::Instant};

use binviz::{
    calculate_entropy_histogram, calculate_histogram, display_most_frequent, full_analysis,
    generate_color_image, generate_image,
};

#[derive(Debug, Clone, Subcommand)]
enum CliCommand {
    /// Calculate the n-dimensional entropy of a given file, for n in 1..=count, in bits per `n` bytes.
    Entropy {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long)]
        count: usize,
    },
    /// Get the bytes in sorted order according to their frequency of a given file.
    Frequency {
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Visualize the given file as an image (digraph analysis).
    ///
    /// We scan pair of bytes from the file and treat that as x and y coordinates into the image.
    /// The pixels brightness will correspond with how many occurrences we have.
    ///
    /// This can show conditional relationships within a binary file.
    /// Distinct file formats will produce distinct recognizable patterns in the image.
    Visualize {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long)]
        trigraph: bool,
    },
    /// Perform a full analysis, by performing all other commands on every file
    /// and collecting the output into folders corresponding to each file.
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
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Cli::parse();
    match args.command {
        CliCommand::Entropy { file, count } => {
            info!("start: executing entropy subcommand...");
            let start_entropy_command = Instant::now();
            info!("start: initializing empty table with headers...");
            let start_table = Instant::now();
            let mut table = Table::new();
            table.load_preset(ASCII_MARKDOWN);
            table.set_header(["Dimension", "Entropy", "Relative Entropy"]);
            let elapsed_table = start_table.elapsed();
            info!(
                "end: finished initializing empty table with headers, with elapsed time: {:?}",
                elapsed_table
            );
            info!("start: calculating the actual entries of the table...");
            let start_collecting = Instant::now();
            for i in 1..=count {
                info!("start: calculating histogram of dimension `{}`...", i);
                let start_histogram = Instant::now();
                let histogram = calculate_histogram(&file, i);
                let elapsed_histogram = start_histogram.elapsed();
                info!(
                    "end: finished calculating histogram of dimension `{}`, with elapsed time: {:?}",
                    i, elapsed_histogram
                );
                info!("start: calculating entropy of histogram...");
                let start_calc_entropy = Instant::now();
                let entropy = calculate_entropy_histogram(&histogram);
                let elapsed_calc_entropy = start_calc_entropy.elapsed();
                info!(
                    "end: finished calculating entropy of histogram, with elapsed time: {:?}",
                    elapsed_calc_entropy
                );
                info!(
                    "start: additionally calculating relative entropy and adding entry to table..."
                );
                let start_entry_add = Instant::now();
                let rel_entropy = entropy / (8.0f64 * (i as f64));
                table.add_row([
                    format!("{}", i),
                    format!("{:.5} (bits per {} byte(s))", entropy, i),
                    format!("{:.5}", rel_entropy),
                ]);
                let elapsed_entry_add = start_entry_add.elapsed();
                info!("end: finished calculating relative entropy and adding entry to table, with elapsed time: {:?}", elapsed_entry_add);
            }
            let elapsed_collecting = start_collecting.elapsed();
            info!(
                "end: finished collecting the actual entries of the table, with elapsed time: {:?}",
                elapsed_collecting
            );
            let elapsed_entropy_command = start_entropy_command.elapsed();
            info!(
                "end: finished executing entropy subcommand, with elapsed time: {:?}",
                elapsed_entropy_command
            );
            println!("{}", table);
        }
        CliCommand::Frequency { file } => {
            info!("start: executing frequency subcommand...");
            let start_freq_command = Instant::now();

            info!("start: calculating histogram...");
            let start_histogram = Instant::now();
            let histogram = calculate_histogram(&file, 1);
            let elapsed_histogram = start_histogram.elapsed();
            info!(
                "end: finished calculating histogram, with elapsed time: {:?}",
                elapsed_histogram
            );
            let elapsed_freq_command = start_freq_command.elapsed();
            info!(
                "end: finished executing frequency subcommand, with elapsed time: {:?}",
                elapsed_freq_command
            );
            println!("{}", display_most_frequent(&histogram));
        }
        CliCommand::Visualize { file, trigraph } => {
            info!("start: executing visualize subcommand...");
            let start_vis_command = Instant::now();
            if trigraph {
                info!("calculating histogram...");
                let trihistogram = calculate_histogram(&file, 3);
                info!("finished calculating histogram.");
                info!("generating image...");
                let (image, total, avg_total) = generate_color_image(&trihistogram);
                info!("finished generating image.");
                info!("saving image to `.\\output.png`...");
                image.save("output.png").expect("Couldn't save image");
                info!("image saved to '.\\output.png'.");
                info!("`{}` byte pairs visualized.", total);
                info!(
                    "full brightness means `{:.4}` byte pairs at that location.",
                    avg_total
                );
                let elapsed_vis_command = start_vis_command.elapsed();
                info!(
                    "end: finished executing visualize subcommand, with elapsed time: {:?}",
                    elapsed_vis_command
                );
            } else {
                let dihistogram = calculate_histogram(&file, 2);
                info!("finished calculating histogram.");
                info!("generating image...");
                let (image, total, avg_total) = generate_image(&dihistogram);
                info!("finished generating image.");
                info!("saving image to `.\\output.png`...");
                image.save("output.png").expect("Couldn't save image");
                info!("image saved to '.\\output.png'.");
                info!("`{}` byte pairs visualized.", total);
                info!(
                    "full brightness means `{:.4}` byte pairs at that location.",
                    avg_total
                );
                let elapsed_vis_command = start_vis_command.elapsed();
                info!(
                    "end: finished executing visualize subcommand, with elapsed time: {:?}",
                    elapsed_vis_command
                );
            };
        }
        CliCommand::Full { files } => full_analysis(files),
    }
}
