//! Existing desktop scene import delegates to the shared native DOCX parser.
use kraqel_domain::EditorDocument;
use std::path::Path;

pub(super) fn read_file(path: &Path) -> Result<(String, EditorDocument), ()> {
    kraqel_export::read_docx_scene_file(path).map_err(|_| ())
}

#[cfg(test)]
use std::io::Cursor;
#[cfg(test)]
const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
#[cfg(test)]
const CT: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
#[cfg(test)]
const REL: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

#[cfg(test)]
pub(super) fn fixture(body: &str, styles: Option<&str>, extra: &[(&str, &[u8])]) -> Vec<u8> {
    use std::io::Write as _;
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let main = format!("<w:document xmlns:w=\"{W}\"><w:body>{body}</w:body></w:document>");
    let content_types = format!(
        "<Types xmlns=\"{CT}\"><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>"
    );
    let rels = format!(
        "<Relationships xmlns=\"{REL}\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/></Relationships>"
    );
    let mut entries = vec![
        ("[Content_Types].xml", content_types.as_bytes()),
        ("_rels/.rels", rels.as_bytes()),
        ("word/document.xml", main.as_bytes()),
    ];
    if let Some(styles) = styles {
        entries.push(("word/styles.xml", styles.as_bytes()));
    }
    entries.extend_from_slice(extra);
    for (name, data) in entries {
        zip.start_file(name, options).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
