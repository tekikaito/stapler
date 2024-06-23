use clap::{ Arg, Command };

pub fn parse_cli_arguments() -> std::result::Result<
    (std::vec::Vec<std::string::String>, std::string::String),
    &'static str
> {
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

    Ok((input_files, output_file))
}
