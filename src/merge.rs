use lopdf::{ Bookmark, Document, Object, ObjectId };
use std::{ collections::BTreeMap, iter::Map, path };

pub struct LoadedDocument {
    pub pdf_document: Document,
    pub original_filename: String,
    pub filepath: String,
    pub objects: BTreeMap<(u32, u16), Object>,
    pub pages: BTreeMap<ObjectId, Object>,
    pub bookmark: Bookmark,
}

impl LoadedDocument {
    pub fn from_filepath(filepath: &str) -> LoadedDocument {
        let pdf_document = Document::load(filepath).expect("Failed to load PDF file");
        let original_filename = filepath.split(path::MAIN_SEPARATOR).last().unwrap().to_string();

        let mut objects = BTreeMap::new();
        let mut pages = BTreeMap::new();

        let mut first_page_id = (0, 0);
        let mut has_visited_first_page = false;

        let pages_from_doc = pdf_document
            .get_pages()
            .into_values()
            .map(|object_id: (u32, u16)| {
                if !has_visited_first_page {
                    first_page_id = object_id;
                    has_visited_first_page = true;
                }
                (object_id, pdf_document.get_object(object_id).unwrap().to_owned())
            })
            .collect::<BTreeMap<ObjectId, Object>>();

        let bookmark = Bookmark::new(original_filename.clone(), [0.0, 0.0, 1.0], 0, first_page_id);
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
    }

    pub fn from_filepaths(
        filepaths: Vec<&str>
    ) -> Map<std::vec::IntoIter<&str>, fn(&str) -> LoadedDocument> {
        filepaths.into_iter().map(LoadedDocument::from_filepath)
    }
}

pub fn merge_pdfs(
    input_files: Vec<&str>,
    output_file: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Collect all Documents Objects grouped by a map
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version("1.5");

    // Catalog and Pages are mandatory
    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    let loaded_documents: Vec<LoadedDocument> =
        LoadedDocument::from_filepaths(input_files).collect();

    for loaded_document in loaded_documents.into_iter() {
        document.add_bookmark(loaded_document.bookmark, None);
        documents_pages.extend(loaded_document.pages);
        documents_objects.extend(loaded_document.objects);
    }

    // Process all objects except "Page" type
    for (object_id, object) in documents_objects.iter() {
        // We have to ignore "Page" (as are processed later), "Outlines" and "Outline" objects
        // All other objects should be collected and inserted into the main Document
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                // Collect a first "Catalog" object and use it for the future "Pages"
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object { id } else { *object_id },
                    object.clone(),
                ));
            }
            "Pages" => {
                // Collect and update a first "Pages" object and use it for the future "Catalog"
                // We have also to merge all dictionaries of the old and the new "Pages" object
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref object)) = pages_object {
                        if let Ok(old_dictionary) = object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }

                    pages_object = Some((
                        if let Some((id, _)) = pages_object { id } else { *object_id },
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            "Page" => {} // Ignored, processed later and separately
            "Outlines" => {} // Ignored, not supported yet
            "Outline" => {} // Ignored, not supported yet
            _ => {
                document.objects.insert(*object_id, object.clone());
            }
        }
    }

    // If no "Pages" object found abort
    if pages_object.is_none() {
        println!("Pages root not found.");

        return Ok(());
    }

    // Iterate over all "Page" objects and collect into the parent "Pages" created before
    for (object_id, object) in documents_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.as_ref().unwrap().0);

            document.objects.insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    // If no "Catalog" found abort
    if catalog_object.is_none() {
        println!("Catalog root not found.");

        return Ok(());
    }

    let catalog_object = catalog_object.unwrap();
    let pages_object = pages_object.unwrap();

    // Build a new "Pages" with updated fields
    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();

        // Set new pages count
        dictionary.set("Count", documents_pages.len() as u32);

        // Set new "Kids" list (collected from documents pages) for "Pages"
        dictionary.set(
            "Kids",
            documents_pages.into_keys().map(Object::Reference).collect::<Vec<_>>()
        );

        document.objects.insert(pages_object.0, Object::Dictionary(dictionary));
    }

    // Build a new "Catalog" with updated fields
    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs

        document.objects.insert(catalog_object.0, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_object.0);

    // Update the max internal ID as wasn't updated before due to direct objects insertion
    document.max_id = document.objects.len() as u32;

    // Reorder all new Document objects
    document.renumber_objects();

    //Set any Bookmarks to the First child if they are not set to a page
    document.adjust_zero_pages();

    //Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
    if let Some(n) = document.build_outline() {
        if let Ok(Object::Dictionary(ref mut dict)) = document.get_object_mut(catalog_object.0) {
            dict.set("Outlines", Object::Reference(n));
        }
    }

    document.compress();
    document.save(output_file)?;

    Ok(())
}
