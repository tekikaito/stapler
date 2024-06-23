mod merge;

use clap::{ Arg, Command };
use merge::merge_pdfs;

fn main() {
    let matches = Command::new("stapler")
        .version("1.0")
        .author("Marc Gilbrecht <marc-gilbrecht@outlook.de>")
        .about("Merges multiple PDFs into one")
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
                .alias("o")
                .long("output")
                .value_name("FILE")
                .help("Output PDF file")
                .required(true)
        )
        .get_matches();

    let input_files = matches.get_many::<String>("input").unwrap();
    let input_files: Vec<&str> = input_files
        .into_iter()
        .map(|s| s.as_str())
        .collect();
    if input_files.len() < 2 {
        eprintln!("You need to provide at least two input files");
        std::process::exit(1);
    }

    let output_file = matches.get_one::<String>("output").unwrap();

    merge_pdfs(input_files, output_file).expect("Failed to merge PDFs");
    println!("PDFs merged successfully. Output file: {}", output_file);
}
