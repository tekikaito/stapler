use stapler::stapler;
use stapler::merge::FileSystemOptions;
use stapler::cli::parse_cli_arguments;

fn main() {
    if let Ok((input_files, output_file)) = parse_cli_arguments() {
        let file_options = FileSystemOptions::from((&input_files, &output_file));

        println!("[STAPLER] Merging PDFs: {:?} into {}", input_files, output_file);

        if let Err(e) = stapler(file_options) {
            eprintln!("[STAPLER] Error: {}", e);
            std::process::exit(1);
        }

        println!("[STAPLER] PDFs merged successfully. Output file: {}", output_file);
    }
}
