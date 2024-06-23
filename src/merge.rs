use lopdf::{ Bookmark, Document, Object, ObjectId };
use std::{ collections::BTreeMap, path::MAIN_SEPARATOR, result::Result };

pub(crate) struct FileSystemMergingDestination<'a> {
    pub output_file: &'a str,
}

pub(crate) struct FileSystemMergingSource<'a> {
    pub input_file: &'a str,
}

pub(crate) trait FileSystemDocumentWriter {
    fn write_document(
        &self,
        document: Document,
        output_file: &str
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub(crate) struct FileSystemOptions<'a> {
    pub input_sources: Vec<FileSystemMergingSource<'a>>,
    pub destination: FileSystemMergingDestination<'a>,
}

impl DocumentLoader for FileSystemMergingSource<'_> {
    fn load(&self) -> MergableDocument {
        let pdf_document = Document::load(self.input_file).expect("Failed to load PDF file");
        let original_filename = self.input_file.split(MAIN_SEPARATOR).last().unwrap().to_string();

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

        let bookmark = Bookmark::new(original_filename.clone(), [0.0, 0.0, 1.0], 0, first_page_id);

        pages.extend(pages_from_doc);
        objects.extend(pdf_document.objects.clone());

        MergableDocument {
            pdf_document,
            original_filename,
            filepath: self.input_file.to_string(),
            objects,
            pages,
            bookmark,
        }
    }
}

fn update_document_hierarchy(
    document: &mut Document,
    root_page: (ObjectId, Object),
    catalog_object: (ObjectId, Object),
    documents_pages: BTreeMap<ObjectId, Object>
) {
    if let Ok(dictionary) = root_page.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", documents_pages.len() as u32);
        dictionary.set(
            "Kids",
            documents_pages
                .keys()
                .map(|arg0: &(u32, u16)| Object::Reference(*arg0))
                .collect::<Vec<_>>()
        );
        document.objects.insert(root_page.0, Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", root_page.0);
        dictionary.remove(b"Outlines");
        document.objects.insert(catalog_object.0, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_object.0);
    document.max_id = document.objects.len() as u32;
    document.renumber_objects();
    document.adjust_zero_pages();

    if let Some(n) = document.build_outline() {
        if let Ok(Object::Dictionary(ref mut dict)) = document.get_object_mut(catalog_object.0) {
            dict.set("Outlines", Object::Reference(n));
        }
    }
}

type ProcessedObjectsResult = Result<
    (((u32, u16), lopdf::Object), ((u32, u16), lopdf::Object)),
    &'static str
>;

fn process_documents_objects(
    document: &mut Document,
    documents_objects: &BTreeMap<ObjectId, Object>
) -> ProcessedObjectsResult {
    let mut root_catalog_object: Option<(ObjectId, Object)> = None;
    let mut root_page_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects {
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                root_catalog_object.get_or_insert((*object_id, object.clone()));
            }
            "Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref existing_object)) = root_page_object {
                        if let Ok(existing_dict) = existing_object.as_dict() {
                            dictionary.extend(&existing_dict.clone());
                        }
                    }
                    root_page_object = Some((*object_id, Object::Dictionary(dictionary)));
                }
            }
            "Page" | "Outlines" | "Outline" => {}
            _ => {
                document.objects.insert(*object_id, object.clone());
            }
        }
    }

    if root_page_object.is_none() {
        return Err("Pages root not found.");
    }

    if root_catalog_object.is_none() {
        return Err("Catalog root not found.");
    }

    Ok((root_page_object.unwrap(), root_catalog_object.unwrap()))
}

fn insert_pages(document: &mut Document, pages: BTreeMap<ObjectId, Object>, parent: (u32, u16)) {
    for (object_id, object) in pages {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", Object::Reference(parent));
            document.objects.insert(object_id, Object::Dictionary(dictionary));
        }
    }
}

pub(crate) fn process_mergable_documents(
    document: &mut Document,
    documents: Vec<MergableDocument>
) -> Result<(), Box<dyn std::error::Error>> {
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();

    for loaded_document in documents {
        document.add_bookmark(loaded_document.bookmark, None);
        documents_pages.extend(loaded_document.pages);
        documents_objects.extend(loaded_document.objects);
    }

    if
        let Ok((root_catalog_object, root_page_object)) = process_documents_objects(
            document,
            &documents_objects
        )
    {
        insert_pages(document, documents_pages.clone(), root_page_object.0);
        update_document_hierarchy(document, root_page_object, root_catalog_object, documents_pages);
    }

    Ok(())
}
