pub mod merge;

use std::fs::File;

use anyhow::{Context, Result};
use merge::loader::{DocumentLoader, MergableDocument};
use merge::{merge_documents, FileSystemOptions};

pub fn stapler(options: FileSystemOptions) -> Result<File> {
    let loaded_documents = options
        .input_sources
        .iter()
        .map(|source| source.load_from())
        .collect::<Vec<MergableDocument>>();

    let mut document = merge_documents(loaded_documents, options.compress)?;
    document
        .save(options.destination.output_file)
        .context("Failed to save output file")
}

