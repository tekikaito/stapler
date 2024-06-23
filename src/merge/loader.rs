use std::collections::BTreeMap;

use lopdf::{ Bookmark, Object, ObjectId };

pub(crate) trait DocumentLoader {
    fn load(&self) -> MergableDocument;
}

pub(crate) struct MergableDocument {
    pub objects: BTreeMap<ObjectId, Object>,
    pub pages: BTreeMap<ObjectId, Object>,
    pub bookmark: Bookmark,
}

pub mod fs {
    use std::{ collections::BTreeMap, path::MAIN_SEPARATOR };
    use lopdf::{ Bookmark, Document, Object, ObjectId };

    use crate::DocumentLoader;
    use super::MergableDocument;

    pub(crate) struct FileSystemMergingDestination<'a> {
        pub output_file: &'a str,
    }

    pub(crate) struct FileSystemMergingSource<'a> {
        pub input_file: &'a str,
    }

    impl DocumentLoader for FileSystemMergingSource<'_> {
        fn load(&self) -> MergableDocument {
            let pdf_document = Document::load(self.input_file).expect("Failed to load PDF file");
            let original_filename = self.input_file
                .split(MAIN_SEPARATOR)
                .last()
                .unwrap()
                .to_string();

            let mut objects = BTreeMap::new();
            let mut pages = BTreeMap::new();
            let mut first_page_id = (0, 0);
            let mut has_visited_first_page = false;

            let pages_from_doc = pdf_document
                .get_pages()
                .into_values()
                .map(|object_id| {
                    if !has_visited_first_page {
                        first_page_id = object_id;
                        has_visited_first_page = true;
                    }
                    (object_id, pdf_document.get_object(object_id).unwrap().clone())
                })
                .collect::<BTreeMap<ObjectId, Object>>();

            let bookmark: Bookmark = Bookmark::new(
                original_filename.clone(),
                [0.0, 0.0, 1.0],
                0,
                first_page_id
            );

            pages.extend(pages_from_doc);
            objects.extend(pdf_document.objects.clone());

            MergableDocument {
                objects,
                pages,
                bookmark,
            }
        }
    }
}
