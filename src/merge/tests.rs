use lopdf::{ content::{ Content, Operation }, dictionary, Stream };

use super::*;

// Function to create a sample PDF document
pub fn create_sample_pdf(title: &str) -> Document {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(
        dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
    }
    );
    let resources_id = doc.add_object(
        dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    }
    );
    let content = Content {
        operations: vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), (48).into()]),
            Operation::new("Td", vec![(100).into(), (600).into()]),
            Operation::new("Tj", vec![Object::string_literal(title)]),
            Operation::new("ET", vec![])
        ],
    };
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page_id = doc.add_object(
        dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "Contents" => content_id,
        "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    }
    );
    let pages =
        dictionary! {
        "Type" => "Pages",
        "Kids" => vec![page_id.into()],
        "Count" => 1,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages));
    let catalog_id = doc.add_object(
        dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    }
    );
    doc.trailer.set("Root", catalog_id);

    doc
}

#[test]
fn test_merge_two_documents() {
    let doc1 = create_sample_pdf("Document 1");
    let doc2 = create_sample_pdf("Document 2");

    let mergable_docs = vec![
        MergableDocument::from_document("doc1.pdf", doc1),
        MergableDocument::from_document("doc2.pdf", doc2)
    ];

    let result = merge_documents(mergable_docs);

    assert!(result.is_ok());
    let merged_doc = result.unwrap();

    // Check if the merged document has the expected number of pages
    let num_pages = merged_doc.get_pages().len();
    assert_eq!(num_pages, 2, "Merged document should have 2 pages");
}

#[test]
fn test_merge_empty_documents() {
    let mergable_docs = vec![];

    let result = merge_documents(mergable_docs);

    assert!(result.is_err());
    let error = result.err().unwrap();
    assert_eq!(error.to_string(), "At least two documents are required to merge.");
}
