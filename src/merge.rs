use lopdf::{ Bookmark, Document, Object, ObjectId };
use std::{ collections::BTreeMap, path::MAIN_SEPARATOR };

pub struct LoadedDocument {
    pub pdf_document: Document,
    pub original_filename: String,
    pub filepath: String,
    pub objects: BTreeMap<ObjectId, Object>,
    pub pages: BTreeMap<ObjectId, Object>,
    pub bookmark: Bookmark,
}

pub struct FileSystemOptions<'a> {
    pub input_files: Vec<&'a str>,
    pub output_file: &'a str,
}

trait DocumentBatchLoader {
    fn load_documents(&self) -> Vec<LoadedDocument>;
}

trait CompressingDocumentWriter {
    fn write_document(
        &self,
        document: Document,
        output_file: &str
    ) -> Result<(), Box<dyn std::error::Error>>;
}

impl DocumentBatchLoader for FileSystemOptions<'_> {
    fn load_documents(&self) -> Vec<LoadedDocument> {
        self.input_files
            .iter()
            .map(|&filepath| {
                let pdf_document = Document::load(filepath).expect("Failed to load PDF file");
                let original_filename = filepath.split(MAIN_SEPARATOR).last().unwrap().to_string();

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

                let bookmark = Bookmark::new(
                    original_filename.clone(),
                    [0.0, 0.0, 1.0],
                    0,
                    first_page_id
                );

                pages.extend(pages_from_doc);
                objects.extend(pdf_document.objects.clone());

                LoadedDocument {
                    pdf_document,
                    original_filename,
                    filepath: filepath.to_string(),
                    objects,
                    pages,
                    bookmark,
                }
            })
            .collect()
    }
}

impl CompressingDocumentWriter for FileSystemOptions<'_> {
    fn write_document(
        &self,
        mut document: Document,
        output_file: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        document.compress();
        document.save(output_file)?;
        Ok(())
    }
}

pub fn merge_pdfs(options: FileSystemOptions) -> Result<(), Box<dyn std::error::Error>> {
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version("1.5");

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    let loaded_documents = options.load_documents();

    for loaded_document in loaded_documents {
        document.add_bookmark(loaded_document.bookmark, None);
        documents_pages.extend(loaded_document.pages);
        documents_objects.extend(loaded_document.objects);
    }

    for (object_id, object) in &documents_objects {
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                catalog_object.get_or_insert((*object_id, object.clone()));
            }
            "Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref existing_object)) = pages_object {
                        if let Ok(existing_dict) = existing_object.as_dict() {
                            dictionary.extend(&existing_dict.clone());
                        }
                    }
                    pages_object = Some((*object_id, Object::Dictionary(dictionary)));
                }
            }
            "Page" | "Outlines" | "Outline" => {}
            _ => {
                document.objects.insert(*object_id, object.clone());
            }
        }
    }

    if pages_object.is_none() {
        return Err("Pages root not found.".into());
    }

    if catalog_object.is_none() {
        return Err("Catalog root not found.".into());
    }

    for (object_id, object) in documents_pages.clone() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.as_ref().unwrap().0);
            document.objects.insert(object_id, Object::Dictionary(dictionary));
        }
    }

    let catalog_object = catalog_object.unwrap();
    let pages_object = pages_object.unwrap();

    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", documents_pages.clone().len() as u32);
        dictionary.set(
            "Kids",
            documents_pages.into_keys().map(Object::Reference).collect::<Vec<_>>()
        );
        document.objects.insert(pages_object.0, Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
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

    options.write_document(document, options.output_file)?;

    Ok(())
}
