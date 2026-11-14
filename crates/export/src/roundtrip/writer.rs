use super::{
    inventory::{self, CP, PROPERTY, Role, VT, W},
    package::{self, OFFICE, REL},
};
use crate::{Inline, Manuscript};
use std::{collections::BTreeMap, fmt::Write};

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\r', "&#13;")
}
fn style(role: Role) -> String {
    if let Some(level) = role.heading() {
        return format!("Heading{level}");
    }
    match role {
        Role::ProjectTitle => "Title",
        Role::ChapterTitle => "ChapterTitle",
        Role::SceneTitle => "SceneTitle",
        _ => "Normal",
    }
    .to_owned()
}
fn alias(role: Role) -> &'static str {
    match role {
        Role::ProjectTitle => "Projekttitel",
        Role::ChapterTitle => "Kapiteltitel",
        Role::SceneTitle => "Szenentitel",
        Role::Paragraph => "Absatz",
        _ => "Überschrift",
    }
}
fn runs(output: &mut String, values: &[Inline]) -> Result<(), ()> {
    for inline in values {
        match inline {
            Inline::HardBreak => output.push_str("<w:r><w:br/></w:r>"),
            Inline::Text { text, bold, italic } => {
                output.push_str("<w:r>");
                if *bold || *italic {
                    output.push_str("<w:rPr>");
                    if *bold {
                        output.push_str("<w:b/>");
                    }
                    if *italic {
                        output.push_str("<w:i/>");
                    }
                    output.push_str("</w:rPr>");
                }
                write!(
                    output,
                    "<w:t xml:space=\"preserve\">{}</w:t></w:r>",
                    escape(text)
                )
                .map_err(|_| ())?;
            }
        }
    }
    Ok(())
}

pub(super) fn properties(nodes: Vec<inventory::Node>) -> Result<(String, String), ()> {
    let (payload, header) = inventory::payload(nodes)?;
    let mut props = format!("<Properties xmlns=\"{CP}\" xmlns:vt=\"{VT}\">");
    let header_json = serde_json::to_string(&header).map_err(|_| ())?;
    let mut values = vec![(PROPERTY.to_owned(), header_json)];
    for (index, chunk) in payload.as_bytes().chunks(inventory::CHUNK).enumerate() {
        values.push((
            format!("{PROPERTY}.{:04}", index + 1),
            std::str::from_utf8(chunk).map_err(|_| ())?.to_owned(),
        ));
    }
    for (index, (key, value)) in values.iter().enumerate() {
        write!(props,"<property fmtid=\"{{D5CDD505-2E9C-101B-9397-08002B2CF9AE}}\" pid=\"{}\" name=\"{key}\"><vt:lpwstr>{}</vt:lpwstr></property>",index+2,escape(value)).map_err(|_|())?;
    }
    props.push_str("</Properties>");
    Ok((props, header.sha256))
}

pub(super) fn write(manuscript: &Manuscript) -> Result<Vec<u8>, ()> {
    let records = inventory::records(manuscript);
    let nodes = records.iter().map(|r| r.node.clone()).collect::<Vec<_>>();
    inventory::topology(&nodes)?;
    let (props, digest) = properties(nodes)?;
    let indices: BTreeMap<_, _> = records
        .iter()
        .enumerate()
        .map(|(i, r)| (r.node.id.as_str(), i))
        .collect();
    let mut document = format!("<w:document xmlns:w=\"{W}\"><w:body>");
    for (index, record) in records.iter().enumerate() {
        let Some(values) = &record.runs else {
            continue;
        };
        write!(
            document,
            "<w:p><w:pPr><w:pStyle w:val=\"{}\"/>",
            style(record.node.role)
        )
        .map_err(|_| ())?;
        if let Some(id) = record.numbering {
            write!(
                document,
                "<w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"{id}\"/></w:numPr>"
            )
            .map_err(|_| ())?;
        }
        let mut lists = 0;
        let mut ancestor = index;
        while let Some(parent) = indices.get(records[ancestor].node.parent.as_str()) {
            ancestor = *parent;
            if matches!(
                records[ancestor].node.role,
                Role::BulletList | Role::OrderedList
            ) {
                lists += 1;
            }
        }
        let indent = record.quote * 360
            + if lists > 0 {
                720 + (lists - 1) * 360
            } else {
                0
            };
        if indent > 0 {
            write!(document, "<w:ind w:left=\"{indent}\"/>").map_err(|_| ())?;
        }
        write!(document,"</w:pPr><w:sdt><w:sdtPr><w:alias w:val=\"{}\"/><w:tag w:val=\"{}\"/><w:id w:val=\"{}\"/><w:richText/></w:sdtPr><w:sdtContent>",alias(record.node.role),record.node.role.marker(&record.node.id,&digest),index+1).map_err(|_|())?;
        runs(&mut document, values)?;
        document.push_str("</w:sdtContent></w:sdt></w:p>");
    }
    document.push_str("<w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/><w:pgMar w:top=\"1134\" w:right=\"1134\" w:bottom=\"1134\" w:left=\"1134\"/></w:sectPr></w:body></w:document>");
    let styles = styles()?;
    let numbers = numbers(&records)?;
    let root_rels = format!(
        "<Relationships xmlns=\"{REL}\"><Relationship Id=\"rId1\" Type=\"{OFFICE}officeDocument\" Target=\"word/document.xml\"/><Relationship Id=\"rId2\" Type=\"{OFFICE}custom-properties\" Target=\"docProps/custom.xml\"/></Relationships>"
    );
    let document_rels = format!(
        "<Relationships xmlns=\"{REL}\"><Relationship Id=\"rId1\" Type=\"{OFFICE}styles\" Target=\"styles.xml\"/><Relationship Id=\"rId2\" Type=\"{OFFICE}numbering\" Target=\"numbering.xml\"/></Relationships>"
    );
    package::write(&BTreeMap::from([
        ("_rels/.rels".to_owned(), root_rels),
        ("word/_rels/document.xml.rels".to_owned(), document_rels),
        ("word/document.xml".to_owned(), document),
        ("word/styles.xml".to_owned(), styles),
        ("word/numbering.xml".to_owned(), numbers),
        ("docProps/custom.xml".to_owned(), props),
    ]))
}

fn styles() -> Result<String, ()> {
    let mut styles = format!("<w:styles xmlns:w=\"{W}\">");
    let definitions = [
        ("Normal", 22, None),
        ("Title", 36, None),
        ("ChapterTitle", 30, None),
        ("SceneTitle", 26, None),
        ("Heading1", 28, Some(0)),
        ("Heading2", 26, Some(1)),
        ("Heading3", 24, Some(2)),
        ("Heading4", 23, Some(3)),
        ("Heading5", 22, Some(4)),
        ("Heading6", 22, Some(5)),
    ];
    for (id, size, outline) in definitions {
        write!(
            styles,
            "<w:style w:type=\"paragraph\" w:styleId=\"{id}\"{}><w:name w:val=\"{id}\"/>",
            if id == "Normal" {
                " w:default=\"1\""
            } else {
                ""
            }
        )
        .map_err(|_| ())?;
        if id != "Normal" {
            styles.push_str("<w:basedOn w:val=\"Normal\"/>");
        }
        styles.push_str("<w:pPr><w:spacing w:after=\"100\"/>");
        if let Some(outline) = outline {
            write!(styles, "<w:outlineLvl w:val=\"{outline}\"/>").map_err(|_| ())?;
        }
        write!(styles,"</w:pPr><w:rPr><w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\"/><w:color w:val=\"000000\"/><w:sz w:val=\"{size}\"/></w:rPr></w:style>").map_err(|_|())?;
    }
    styles.push_str("</w:styles>");
    Ok(styles)
}

fn numbers(records: &[inventory::Record]) -> Result<String, ()> {
    let mut numbers = format!("<w:numbering xmlns:w=\"{W}\">");
    let lists: Vec<_> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| matches!(r.node.role, Role::BulletList | Role::OrderedList))
        .collect();
    for (index, record) in &lists {
        let id = index + 1;
        let (format, marker) = if record.node.role == Role::BulletList {
            ("bullet", "•")
        } else {
            ("decimal", "%1.")
        };
        write!(numbers,"<w:abstractNum w:abstractNumId=\"{id}\"><w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"{format}\"/><w:lvlText w:val=\"{marker}\"/><w:lvlJc w:val=\"left\"/><w:pPr><w:ind w:left=\"720\" w:hanging=\"360\"/></w:pPr></w:lvl></w:abstractNum>").map_err(|_|())?;
    }
    for (index, _) in lists {
        let id = index + 1;
        write!(
            numbers,
            "<w:num w:numId=\"{id}\"><w:abstractNumId w:val=\"{id}\"/></w:num>"
        )
        .map_err(|_| ())?;
    }
    numbers.push_str("</w:numbering>");
    Ok(numbers)
}
