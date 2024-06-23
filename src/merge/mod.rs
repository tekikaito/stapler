pub mod loader;

use loader::{ fs::{ FileSystemMergingDestination, FileSystemMergingSource }, MergableDocument };
use lopdf::{ Document, Object, ObjectId };
use std::{ collections::BTreeMap, result::Result };

pub(crate) struct FileSystemOptions<'a> {
    pub input_sources: Vec<FileSystemMergingSource<'a>>,
    pub destination: FileSystemMergingDestination<'a>,
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

pub(crate) fn merge_documents(
    root_document: &mut Document,
    aggregated_documents: Vec<MergableDocument>
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pages = BTreeMap::new();
    let mut objects = BTreeMap::new();

    aggregated_documents.into_iter().for_each(|doc| {
        root_document.add_bookmark(doc.bookmark, None);
        pages.extend(doc.pages);
        objects.extend(doc.objects);
    });

    if let Ok((root_catalog, root_page)) = process_documents_objects(root_document, &objects) {
        insert_pages(root_document, pages.clone(), root_page.0);
        update_document_hierarchy(root_document, root_page, root_catalog, pages);
    }

    Ok(())
}
