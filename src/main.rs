mod merge;
mod cli;

use merge::{ merge_pdfs, FileSystemMergingDestination, FileSystemMergingSource, FileSystemOptions };
use cli::parse_cli_arguments;

fn main() {
    println!("[STAPLER] PDF MERGER");

    if let Ok((input_files, output_file)) = parse_cli_arguments() {
        let file_options = FileSystemOptions {
            input_sources: input_files
                .iter()
                .map(|input_file| FileSystemMergingSource { input_file })
                .collect(),
            destination: FileSystemMergingDestination { output_file: &output_file },
        };

        println!("[STAPLER] Merging PDFs: {:?} into {}", input_files, output_file);

        if let Err(e) = merge_pdfs(file_options) {
            eprintln!("[STAPLER] Error: {}", e);
            std::process::exit(1);
        }

        println!("[STAPLER] PDFs merged successfully. Output file: {}", output_file);
    }
}
