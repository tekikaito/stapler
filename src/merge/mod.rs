pub mod loader;
#[cfg(test)]
pub mod tests;
use anyhow::{Context, Result};
use loader::{
    fs::{FileSystemMergingDestination, FileSystemMergingSource},
    MergableDocument,
};
use lopdf::{Bookmark, Document, Object, ObjectId};
use std::collections::BTreeMap;

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
    pages: BTreeMap<ObjectId, Object>,
) -> Result<()> {
    let root_page_dictionary = {
        let mut dictionary = root_page
            .1
            .as_dict()
            .context("Could not get dictionary from root page object")?
            .clone();
        dictionary.set("Count", pages.len() as u32);
        dictionary.set(
            "Kids",
            pages
                .keys()
                .map(|arg0: &(u32, u16)| Object::Reference(*arg0))
                .collect::<Vec<_>>(),
        );
        dictionary
    };
    document
        .objects
        .insert(root_page.0, Object::Dictionary(root_page_dictionary));

    let catalog_dictionary = {
        let mut dictionary = catalog_object
            .1
            .as_dict()
            .context("Could not get dictionary from catalog object")?
            .clone();
        dictionary.set("Pages", root_page.0);
        dictionary.remove(b"Outlines");
        dictionary
    };
    document
        .objects
        .insert(catalog_object.0, Object::Dictionary(catalog_dictionary));

    document.trailer.set("Root", catalog_object.0);
    document.max_id = document.objects.len() as u32;
    document.renumber_objects();
    document.adjust_zero_pages();

    if let Some(n) = document.build_outline() {
        if let Ok(Object::Dictionary(ref mut dict)) = document.get_object_mut(catalog_object.0) {
            dict.set("Outlines", Object::Reference(n));
        } else {
            anyhow::bail!("Could not get mutable dictionary from catalog object");
        }
    }

    Ok(())
}

struct ProcessedObjects {
    root_page_object: (ObjectId, Object),
    root_catalog_object: (ObjectId, Object),
}

fn process_documents_objects(
    document: &mut Document,
    objects: BTreeMap<ObjectId, Object>,
) -> Result<ProcessedObjects> {
    let mut root_catalog_object: Option<(ObjectId, Object)> = None;
    let mut root_page_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in objects {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                root_catalog_object.get_or_insert((object_id, object));
            }
            b"Pages" => {
                let Object::Dictionary(mut dictionary) = object else {
                    continue;
                };
                if let Some((_, ref existing_object)) = root_page_object {
                    if let Ok(existing_dict) = existing_object.as_dict() {
                        dictionary.extend(existing_dict);
                    }
                }
                root_page_object = Some((object_id, Object::Dictionary(dictionary)));
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                document.objects.insert(object_id, object);
            }
        }
    }

    let root_page_object = root_page_object.context("Pages root not found.")?;
    let root_catalog_object = root_catalog_object.context("Catalog root not found.")?;

    Ok(ProcessedObjects {
        root_page_object,
        root_catalog_object,
    })
}

fn insert_pages(
    document: &mut Document,
    pages: &BTreeMap<ObjectId, Object>,
    parent: ObjectId,
) -> Result<()> {
    for (object_id, object) in pages {
        let page_dict = {
            let mut dict = object
                .as_dict()
                .context("Could not get dictionary from page object.")?
                .clone();
            dict.set("Parent", Object::Reference(parent));
            dict
        };
        document
            .objects
            .insert(*object_id, Object::Dictionary(page_dict));
    }

    Ok(())
}

fn add_bookmarks(doc: &mut Document, bookmarks: &BTreeMap<Option<u32>, Bookmark>) {
    for (reference, bookmark) in bookmarks {
        doc.add_bookmark(bookmark.clone(), *reference);
    }
}

pub fn merge_documents(input_docs: Vec<MergableDocument>, compress: bool) -> Result<Document> {
    anyhow::ensure!(
        input_docs.len() >= 2,
        "At least two documents are required to merge."
    );

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

    let ProcessedObjects {
        root_catalog_object,
        root_page_object,
    } = process_documents_objects(&mut result_doc, objects_map)?;

    add_bookmarks(&mut result_doc, &bookmarks_map);
    insert_pages(&mut result_doc, &pages_map, root_page_object.0)?;
    update_document_hierarchy(
        &mut result_doc,
        root_page_object,
        root_catalog_object,
        pages_map,
    )?;

    if compress {
        result_doc.compress();
    }

    Ok(result_doc)
}
