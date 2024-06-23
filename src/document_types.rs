pub(crate) trait DocumentLoader {
    fn load(&self) -> MergableDocument;
}

pub(crate) struct MergableDocument {
    pub pdf_document: Document,
    pub original_filename: String,
    pub filepath: String,
    pub objects: BTreeMap<ObjectId, Object>,
    pub pages: BTreeMap<ObjectId, Object>,
    pub bookmark: Bookmark,
}
