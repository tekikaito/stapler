mod merge;
mod cli;

use merge::{ merge_pdfs, FileSystemOptions };
use cli::parse_cli_arguments;

fn main() {
    println!("[STAPLER] PDF MERGER");

    if let Ok((input_files, output_file)) = parse_cli_arguments() {
        let file_options = FileSystemOptions {
            input_files: input_files
                .iter()
                .map(|s| s.as_str())
                .collect(),
            output_file: output_file.as_str(),
        };

        println!(
            "[STAPLER] Merging PDFs: {:?} into {}",
            file_options.input_files,
            file_options.output_file
        );

        if let Err(e) = merge_pdfs(file_options) {
            eprintln!("[STAPLER] Error: {}", e);
            std::process::exit(1);
        }

        println!("[STAPLER] PDFs merged successfully. Output file: {}", output_file);
    }
}
