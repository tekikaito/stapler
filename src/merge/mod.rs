pub mod loader;
pub mod tests;
use loader::{ fs::{ FileSystemMergingDestination, FileSystemMergingSource }, MergableDocument };
use lopdf::{ Bookmark, Document, Object, ObjectId };
use std::{ collections::BTreeMap, error::Error, result::Result };

#[derive(Debug, Clone)]
pub struct FileSystemOptions<'a> {
    pub input_sources: Vec<FileSystemMergingSource<'a>>,
    pub destination: FileSystemMergingDestination<'a>,
    pub compress: bool,
}

impl<'a> From<(&'a Vec<String>, &'a String, bool)> for FileSystemOptions<'a> {
    fn from((input_files, output_file, compress): (&'a Vec<String>, &'a String, bool)) -> Self {
        FileSystemOptions {
            input_sources: input_files
                .iter()
                .map(|input_file| FileSystemMergingSource { input_file })
                .collect(),
            destination: FileSystemMergingDestination { output_file },
            compress,
        }
    }
}

fn update_document_hierarchy(
    document: &mut Document,
    root_page: (ObjectId, Object),
    catalog_object: (ObjectId, Object),
    pages: BTreeMap<ObjectId, Object>
) {
    if let Ok(dictionary) = root_page.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", pages.len() as u32);
        dictionary.set(
            "Kids",
            pages
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

type ProcessedObjectsResult = Result<(((u32, u16), Object), ((u32, u16), Object)), &'static str>;

fn process_documents_objects(
    document: &mut Document,
    objects: &BTreeMap<ObjectId, Object>
) -> ProcessedObjectsResult {
    let mut root_catalog_object: Option<(ObjectId, Object)> = None;
    let mut root_page_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in objects {
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

fn insert_pages(document: &mut Document, pages: &BTreeMap<ObjectId, Object>, parent: (u32, u16)) {
    for (object_id, object) in pages.clone() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", Object::Reference(parent));
            document.objects.insert(object_id, Object::Dictionary(dictionary));
        }
    }
}

fn add_bookmarks(doc: &mut Document, bookmarks: &BTreeMap<Option<u32>, Bookmark>) {
    for (reference, bookmark) in bookmarks {
        doc.add_bookmark(bookmark.clone(), *reference);
    }
}

pub fn merge_documents(
    input_docs: Vec<MergableDocument>,
    compress: bool
) -> Result<Document, Box<dyn Error>> {
    if input_docs.len() < 2 {
        return Err("At least two documents are required to merge.".into());
    }

    let mut pages_map = BTreeMap::new();
    let mut objects_map = BTreeMap::new();
    let mut bookmarks_map = BTreeMap::new();
    let mut max_id: u32 = 1;

    let mut result_doc = Document::with_version("1.5");

    for mut doc in input_docs {
        let first_page_id = doc.renumber(max_id).get_first_page_id();
        bookmarks_map.insert(None, doc.get_filename_based_bookmark(first_page_id));
        pages_map.extend(doc.get_pages());
        objects_map.extend(doc.get_objects());
        max_id = doc.get_max_id() + 1;
    }

    if let Ok((root_catalog, root_page)) = process_documents_objects(&mut result_doc, &objects_map) {
        add_bookmarks(&mut result_doc, &bookmarks_map);
        insert_pages(&mut result_doc, &pages_map, root_page.0);
        update_document_hierarchy(&mut result_doc, root_page, root_catalog, pages_map);
    }

    if compress {
        result_doc.compress();
    }

    Ok(result_doc)
}
