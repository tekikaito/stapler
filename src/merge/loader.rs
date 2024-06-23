use std::collections::BTreeMap;
use lopdf::{ Bookmark, Document, Object, ObjectId };

pub(crate) trait DocumentLoader {
    fn load_from(&self) -> MergableDocument;
}

pub(crate) struct MergableDocument {
    original_filename: String,
    pdf: Document,
}

impl MergableDocument {
    pub(crate) fn get_pages(&self) -> BTreeMap<(u32, u16), Object> {
        self.pdf
            .get_pages()
            .into_values()
            .map(|object_id| { (object_id, self.pdf.get_object(object_id).unwrap().clone()) })
            .collect::<BTreeMap<ObjectId, Object>>()
    }

    pub(crate) fn get_objects(&self) -> BTreeMap<(u32, u16), Object> {
        self.pdf.objects.clone()
    }

    pub(crate) fn renumber(&mut self, offset: u32) -> &mut MergableDocument {
        self.pdf.renumber_objects_with(offset);
        self
    }

    pub(crate) fn get_first_page_id(&self) -> ObjectId {
        *self.pdf.get_pages().values().next().unwrap()
    }

    pub fn get_filename_based_bookmark(&self, page_id: ObjectId) -> Bookmark {
        Bookmark::new(self.original_filename.clone(), [0.0, 0.0, 1.0], 0, page_id)
    }

    pub(crate) fn get_max_id(&self) -> u32 {
        self.pdf.max_id
    }
}

pub mod fs {
    use std::path::MAIN_SEPARATOR;
    use lopdf::Document;

    use crate::DocumentLoader;
    use super::MergableDocument;

    pub(crate) struct FileSystemMergingDestination<'a> {
        pub output_file: &'a str,
    }

    pub(crate) struct FileSystemMergingSource<'a> {
        pub input_file: &'a str,
    }

    impl DocumentLoader for FileSystemMergingSource<'_> {
        fn load_from(&self) -> MergableDocument {
            let pdf = Document::load(self.input_file).expect("Failed to load PDF file");
            let original_filename = self.input_file
                .split(MAIN_SEPARATOR)
                .last()
                .unwrap()
                .to_string();

            MergableDocument { pdf, original_filename }
        }
    }
}
