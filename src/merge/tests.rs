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
            Operation::new("ET", vec![]),
            // ---
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), (48).into()]),
            Operation::new("Td", vec![(200).into(), (600).into()]),
            Operation::new("Tj", vec![Object::string_literal(title)]),
            Operation::new("ET", vec![]),
            // ---
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), (48).into()]),
            Operation::new("Td", vec![(300).into(), (600).into()]),
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

#[allow(dead_code)]
fn test_merge_x_documents(documents_num: u16) {
    let mut mergable_docs = vec![];
    for i in 0..documents_num {
        let doc = create_sample_pdf(&format!("Document {}", i));
        mergable_docs.push(MergableDocument::from_document(&format!("doc{}.pdf", i), doc));
    }

    let result = merge_documents(mergable_docs);

    assert!(result.is_ok());
    let merged_doc = result.unwrap();

    // Check if the merged document has the expected number of pages
    let num_pages = merged_doc.get_pages().len();
    assert_eq!(
        num_pages,
        documents_num as usize,
        "Merged document should have {} pages",
        documents_num
    );

    // Check if the merged document has the expected content
    let pages: Vec<ObjectId> = merged_doc
        .get_pages()
        .iter()
        .map(|page| page.1.to_owned())
        .collect();

    for i in 0..documents_num {
        let page = pages[i as usize];
        let content = merged_doc.get_page_content(page).unwrap();
        let content = String::from_utf8(content).unwrap();
        let expected_content = format!("Document {}", i);
        assert!(
            content.contains(&expected_content),
            "Page content should contain '{}'",
            expected_content
        );
    }
}

#[test]
fn test_merge_2_documents() {
    test_merge_x_documents(2);
}

#[test]
fn test_merge_5_documents() {
    test_merge_x_documents(5);
}

#[test]
fn test_merge_10_documents() {
    test_merge_x_documents(10);
}

#[test]
fn test_merge_20_documents() {
    test_merge_x_documents(20);
}

#[test]
fn test_merge_31_documents() {
    test_merge_x_documents(31);
}

#[test]
fn test_merge_40_documents() {
    test_merge_x_documents(40);
}

#[test]
fn test_merge_100_documents() {
    test_merge_x_documents(100);
}

#[test]
fn test_merge_500_documents() {
    test_merge_x_documents(500);
}

#[test]
fn test_merge_1337_documents() {
    test_merge_x_documents(1337)
}
#[test]
fn test_merge_6666_documents() {
    test_merge_x_documents(6666)
}

#[test]
fn test_merge_empty_documents() {
    let mergable_docs = vec![];
    let result = merge_documents(mergable_docs);

    assert!(result.is_err());
    let error = result.err().unwrap();
    assert_eq!(error.to_string(), "At least two documents are required to merge.");
}
