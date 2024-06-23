mod merge;
mod cli;

use lopdf::Document;
use merge::{
    process_mergable_documents,
    FileSystemMergingDestination,
    FileSystemMergingSource,
    FileSystemOptions,
    MergableDocument,
};
use cli::parse_cli_arguments;

fn stapler(options: FileSystemOptions) -> Result<(), String> {
    let loaded_documents = options.input_sources
        .iter()
        .map(|source| source.load())
        .collect::<Vec<MergableDocument>>();

    let mut document = Document::with_version("1.5");
    process_mergable_documents(&mut document, loaded_documents)?;

    options.destination.write_document(document, options.destination.output_file)?;
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

        if let Err(e) = merge_pdfs(file_options) {
            eprintln!("[STAPLER] Error: {}", e);
            std::process::exit(1);
        }

        println!("[STAPLER] PDFs merged successfully. Output file: {}", output_file);
    }
}
