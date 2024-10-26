use std::process::exit;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, Command};
use stapler::merge::FileSystemOptions;
use stapler::stapler;

fn parse_cli_arguments() -> Result<(Vec<String>, String, bool)> {
    let matches = Command::new("stapler")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILES")
                .help("Input PDF files")
                .num_args(2..)
                .value_delimiter(' ')
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output PDF file")
                .required(true),
        )
        .arg(
            Arg::new("compress")
                .action(ArgAction::SetTrue)
                .short('c')
                .long("compress")
                .help("Compress the output PDF file")
                .required(false),
        )
        .get_matches();

    let input_files: Vec<String> = matches
        .get_many::<String>("input")
        .context("No input files provided")?
        .cloned()
        .collect();

    let output_file: String = matches
        .get_one::<String>("output")
        .context("No output file provided")?
        .clone();

    let compress = matches
        .get_one::<bool>("compress")
        .copied()
        .unwrap_or(false);

    Ok((input_files, output_file, compress))
}

fn main() -> Result<()> {
    let (input_files, output_file, compress) = parse_cli_arguments()?;
    let file_options = FileSystemOptions::from((&input_files, &output_file, compress));

    println!(
        "[STAPLER] Merging PDFs: {:?} into {}",
        input_files, output_file
    );

    if let Err(e) = stapler(file_options) {
        eprintln!("[STAPLER] Error: {}", e);
        exit(1);
    }

    println!(
        "[STAPLER] PDFs merged successfully. Output file: {}",
        output_file
    );

    Ok(())
}
