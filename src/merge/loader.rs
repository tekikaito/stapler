use std::collections::BTreeMap;
use lopdf::{ Bookmark, Document, Object, ObjectId };

pub trait DocumentLoader {
    fn load(&self) -> MergableDocument;
}

pub struct MergableDocument {
    original_filename: String,
    pdf: Document,
}

impl MergableDocument {
    pub fn get_pages(&self) -> BTreeMap<(u32, u16), Object> {
        self.pdf
            .get_pages()
            .into_values()
            .map(|object_id| { (object_id, self.pdf.get_object(object_id).unwrap().clone()) })
            .collect::<BTreeMap<ObjectId, Object>>()
    }

    pub fn get_objects(&self) -> BTreeMap<(u32, u16), Object> {
        self.pdf.objects.clone()
    }

    pub fn renumber(&mut self, offset: u32) -> &mut MergableDocument {
        self.pdf.renumber_objects_with(offset);
        self
    }

    pub fn get_first_page_id(&self) -> ObjectId {
        *self.pdf.get_pages().values().next().unwrap()
    }

    pub fn get_filename_based_bookmark(&self, page_id: ObjectId) -> Bookmark {
        Bookmark::new(self.original_filename.clone(), [0.0, 0.0, 1.0], 0, page_id)
    }

    pub fn get_max_id(&self) -> u32 {
        self.pdf.max_id
    }

    pub fn from_document(original_filename: &str, pdf: Document) -> MergableDocument {
        MergableDocument {
            original_filename: original_filename.to_string(),
            pdf,
        }
    }
}

pub mod fs {
    use lopdf::Document;

    use super::*;

    #[derive(Debug, Clone)]
    pub struct FileSystemMergingDestination<'a> {
        pub output_file: &'a str,
    }

    #[derive(Debug, Clone)]
    pub struct FileSystemMergingSource<'a> {
        pub input_file: &'a str,
    }

    impl DocumentLoader for FileSystemMergingSource<'_> {
        fn load(&self) -> MergableDocument {
            let pdf = Document::load(self.input_file).unwrap_or_else(|_|
                panic!("Failed to load {}", self.input_file)
            );
            let original_filename = self.input_file
                .split(std::path::MAIN_SEPARATOR)
                .last()
                .unwrap()
                .to_string();

            MergableDocument { pdf, original_filename }
        }
    }
}
