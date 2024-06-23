use stapler::merge::*;
use lopdf::{ Document, Object, Bookmark };
use std::collections::BTreeMap;

// Helper function to create a sample MergableDocument
fn create_sample_mergable_document() -> MergableDocument {
    let mut pdf_document = Document::with_version("1.5");
    let mut objects = BTreeMap::new();
    let mut pages = BTreeMap::new();

    let page_id = (1, 0);
    let page = Object::Dictionary(Default::default());

    pages.insert(page_id, page.clone());
    objects.insert(page_id, page);

    let bookmark = Bookmark::new("Sample".to_string(), [0.0, 0.0, 1.0], 0, page_id);

    MergableDocument {
        pdf_document,
        original_filename: "sample.pdf".to_string(),
        filepath: "sample.pdf".to_string(),
        objects,
        pages,
        bookmark,
    }
}

#[test]
fn test_process_mergable_documents() {
    let mut document = Document::with_version("1.5");

    let sample_document1 = create_sample_mergable_document();
    let sample_document2 = create_sample_mergable_document();

    let documents = vec![sample_document1, sample_document2];

    let result = process_mergable_documents(&mut document, documents);

    assert!(result.is_ok());

    // Verify that the document now contains the pages and objects from the sample documents
    assert_eq!(document.objects.len(), 4); // 2 pages and 2 bookmarks
    assert!(document.objects.contains_key(&(1, 0)));
}

#[test]
fn test_process_mergable_documents_no_pages() {
    let mut document = Document::with_version("1.5");

    let mut sample_document = create_sample_mergable_document();
    sample_document.pages.clear(); // Remove pages

    let documents = vec![sample_document];

    let result = process_mergable_documents(&mut document, documents);

    assert!(result.is_err()); // Should fail because there are no pages
}

#[test]
fn test_process_mergable_documents_no_catalog() {
    let mut document = Document::with_version("1.5");

    let mut sample_document = create_sample_mergable_document();
    sample_document.objects.remove(&(1, 0)); // Remove catalog

    let documents = vec![sample_document];

    let result = process_mergable_documents(&mut document, documents);

    assert!(result.is_err()); // Should fail because there is no catalog
}
