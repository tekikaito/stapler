mod merge;

use clap::{ Arg, Command };
use merge::{ merge_pdfs, FileSystemOptions };

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
    let output_file = matches.get_one::<String>("output").unwrap();

    let options = FileSystemOptions {
        input_files: input_files
            .into_iter()
            .map(|s| s.as_str())
            .collect(),
        output_file,
    };

    merge_pdfs(options).unwrap();

    println!("PDFs merged successfully. Output file: {}", output_file);
}
