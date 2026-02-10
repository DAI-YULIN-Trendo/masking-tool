use lopdf::Document;
use anyhow::Result;

pub fn load_pdf_from_bytes(data: &[u8]) -> Result<Document> {
    // lopdf automatically handles various PDF versions
    let doc = Document::load_mem(data)?;
    Ok(doc)
}
