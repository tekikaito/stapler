pub mod merge;

use std::fs::File;

use merge::{ FileSystemOptions, merge_documents };
use merge::loader::{ DocumentLoader, MergableDocument };

pub fn stapler(options: FileSystemOptions) -> Result<File, String> {
    let loaded_documents = options.input_sources
        .iter()
        .map(|source| source.load_from())
        .collect::<Vec<MergableDocument>>();

    if let Ok(mut document) = merge_documents(loaded_documents, options.compress) {
        document.save(options.destination.output_file).map_err(|e| e.to_string())
    } else {
        Err("Failed to merge documents".to_string())
    }
}
