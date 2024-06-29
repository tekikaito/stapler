use std::process::exit;

use clap::{ Arg, ArgAction, Command };
use stapler::stapler;
use stapler::merge::FileSystemOptions;

fn parse_cli_arguments() -> Result<(Vec<String>, String, bool), &'static str> {
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
                .required(true)
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output PDF file")
                .required(true)
        )
        .arg(
            Arg::new("compress")
                .action(ArgAction::SetTrue)
                .short('c')
                .long("compress")
                .help("Compress the output PDF file")
                .required(false)
        )
        .get_matches();

    let input_files: Vec<String> = if let Some(files) = matches.get_many::<String>("input") {
        files.map(|s| s.to_owned()).collect()
    } else {
        return Err("No input files provided");
    };

    let output_file: String = if let Some(file) = matches.get_one::<String>("output") {
        file.to_owned()
    } else {
        return Err("No output file provided");
    };

    let compress: bool = matches.get_flag("compress");

    Ok((input_files, output_file, compress))
}

fn main() {
    if let Ok((input_files, output_file, compress)) = parse_cli_arguments() {
        let file_options = FileSystemOptions::from((&input_files, &output_file, compress));

        println!("[STAPLER] Merging PDFs: {:?} into {}", input_files, output_file);

        if let Err(e) = stapler(file_options) {
            eprintln!("[STAPLER] Error: {}", e);
            exit(1);
        }

        println!("[STAPLER] PDFs merged successfully. Output file: {}", output_file);
    }
}
