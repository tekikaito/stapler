mod merge;
mod cli;

use std::fs::File;

use lopdf::Document;
use merge::{
    loader::{
        fs::{ FileSystemMergingDestination, FileSystemMergingSource },
        DocumentLoader,
        MergableDocument,
    },
    merge_documents,
    FileSystemOptions,
};
use cli::parse_cli_arguments;

fn stapler(options: FileSystemOptions) -> Result<File, String> {
    let loaded_documents = options.input_sources
        .iter()
        .map(|source| source.load_from())
        .collect::<Vec<MergableDocument>>();

    let mut document = Document::with_version("1.5");
    if let Err(e) = merge_documents(&mut document, loaded_documents) {
        return Err(e.to_string());
    }

    document.compress();
    document.save(options.destination.output_file).map_err(|e| e.to_string())
}

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

        if let Err(e) = stapler(file_options) {
            eprintln!("[STAPLER] Error: {}", e);
            std::process::exit(1);
        }

        println!("[STAPLER] PDFs merged successfully. Output file: {}", output_file);
    }
}
