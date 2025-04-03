//! Module handling PDF document loading and preparation for merging operations

use std::collections::BTreeMap;
use lopdf::{Bookmark, Document, Object, ObjectId};

/// Trait defining common behavior for loading PDF documents from various sources
pub trait DocumentLoader {
    /// Loads and prepares a document for merging operations
    fn load_from(&self) -> MergableDocument;
}

/// Container for a PDF document with metadata needed for merging operations
pub struct MergableDocument {
    /// Original filename used for bookmarking in merged documents
    original_filename: String,
    /// The loaded PDF document structure
    pdf: Document,
}

impl MergableDocument {
    /// Extracts all pages with their object IDs from the document
    /// Returns BTreeMap<ObjectId, Object> where:
    /// - Key: (generation number, object number) tuple
    /// - Value: Page object
    pub fn get_pages(&self) -> BTreeMap<(u32, u16), Object> {
        self.pdf
            .get_pages()
            .into_values()
            .map(|object_id| {
                (object_id, self.pdf.get_object(object_id).unwrap().clone())
            })
            .collect::<BTreeMap<ObjectId, Object>>()
    }

    /// Retrieves all PDF objects from the document
    /// Used for merging document resources like fonts and images
    pub fn get_objects(&self) -> BTreeMap<(u32, u16), Object> {
        self.pdf.objects.clone()
    }

    /// Renumbers PDF object IDs to prevent conflicts when merging documents
    /// - `offset`: Starting ID for renumbering (should be greater than max ID of previous documents)
    pub fn renumber(&mut self, offset: u32) -> &mut MergableDocument {
        self.pdf.renumber_objects_with(offset);
        self
    }

    /// Gets the object ID of the first page in the document
    /// Used for creating document entry bookmarks
    pub fn get_first_page_id(&self) -> ObjectId {
        *self.pdf.get_pages().values().next().unwrap()
    }

    /// Creates a bookmark using the original filename as title
    /// - `page_id`: Target page for the bookmark
    pub fn get_filename_based_bookmark(&self, page_id: ObjectId) -> Bookmark {
        Bookmark::new(
            self.original_filename.clone(),
            [0.0, 0.0, 1.0], // Default position (upper-left corner)
            0, // Fit display mode
            page_id
        )
    }

    /// Gets the highest object ID in the document
    /// Used to determine renumbering offsets for subsequent documents
    pub fn get_max_id(&self) -> u32 {
        self.pdf.max_id
    }

    /// Constructs a MergableDocument from an existing PDF document
    /// - `original_filename`: Name to use for bookmarks
    /// - `pdf`: Pre-loaded PDF document
    pub fn from_document(original_filename: &str, pdf: Document) -> MergableDocument {
        MergableDocument {
            original_filename: original_filename.to_string(),
            pdf,
        }
    }
}

/// Filesystem-specific document loading implementation
pub mod fs {
    use lopdf::Document;
    use super::*;

    /// Configuration for merged PDF output location
    #[derive(Debug, Clone)]
    pub struct FileSystemMergingDestination<'a> {
        pub output_file: &'a str,
    }

    /// Configuration for PDF input files
    #[derive(Debug, Clone)]
    pub struct FileSystemMergingSource<'a> {
        pub input_file: &'a str,
    }

    impl DocumentLoader for FileSystemMergingSource<'_> {
        /// Loads a PDF document from the filesystem
        /// - Extracts filename for bookmarking
        /// - Handles PDF parsing errors with panic (should be improved for production use)
        fn load_from(&self) -> MergableDocument {
            let pdf = Document::load(self.input_file).unwrap_or_else(|_| 
                panic!("Failed to load {}", self.input_file)
            );
            let original_filename = self.input_file
                .split(std::path::MAIN_SEPARATOR)
                .last() // Extract filename from path
                .unwrap()
                .to_string();

            MergableDocument { pdf, original_filename }
        }
    }
}