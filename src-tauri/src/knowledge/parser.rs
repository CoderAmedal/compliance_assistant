use crate::{AppError, AppResult};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentType {
    Pdf,
    Word,
    Txt,
    Markdown,
    Html,
    Excel,
}

impl DocumentType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "doc" | "docx" => Some(Self::Word),
            "txt" => Some(Self::Txt),
            "md" => Some(Self::Markdown),
            "html" | "htm" => Some(Self::Html),
            "xls" | "xlsx" => Some(Self::Excel),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ParsedDocument {
    pub content: String,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: Option<usize>,
}

pub struct DocumentParser;

impl DocumentParser {
    pub fn parse(path: &Path) -> AppResult<ParsedDocument> {
        let extension = path.extension().and_then(|e| e.to_str()).ok_or_else(|| {
            AppError::DocumentParse("Cannot determine file extension".to_string())
        })?;

        let doc_type = DocumentType::from_extension(extension).ok_or_else(|| {
            AppError::DocumentParse(format!("Unsupported file type: {}", extension))
        })?;

        match doc_type {
            DocumentType::Pdf => Self::parse_pdf(path),
            DocumentType::Word => Self::parse_word(path),
            DocumentType::Txt => Self::parse_txt(path),
            DocumentType::Markdown => Self::parse_markdown(path),
            DocumentType::Html => Self::parse_html(path),
            DocumentType::Excel => Self::parse_excel(path),
        }
    }

    fn parse_txt(path: &Path) -> AppResult<ParsedDocument> {
        let content = std::fs::read_to_string(path)?;

        Ok(ParsedDocument {
            content,
            metadata: DocumentMetadata {
                title: path.file_name().and_then(|n| n.to_str()).map(String::from),
                ..Default::default()
            },
        })
    }

    fn parse_markdown(path: &Path) -> AppResult<ParsedDocument> {
        let content = std::fs::read_to_string(path)?;

        let parser = pulldown_cmark::Parser::new(&content);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);

        Ok(ParsedDocument {
            content: html_output,
            metadata: DocumentMetadata {
                title: path.file_name().and_then(|n| n.to_str()).map(String::from),
                ..Default::default()
            },
        })
    }

    fn parse_html(path: &Path) -> AppResult<ParsedDocument> {
        let content = std::fs::read_to_string(path)?;

        let document = scraper::Html::parse_document(&content);

        let title = document
            .select(&scraper::Selector::parse("title").unwrap())
            .next()
            .map(|el| el.inner_html());

        let body_text: String = document
            .select(&scraper::Selector::parse("body").unwrap())
            .flat_map(|el| el.text())
            .collect();

        Ok(ParsedDocument {
            content: body_text,
            metadata: DocumentMetadata {
                title,
                ..Default::default()
            },
        })
    }

    fn parse_pdf(path: &Path) -> AppResult<ParsedDocument> {
        let pdf = lopdf::Document::load(path)
            .map_err(|e| AppError::DocumentParse(format!("PDF parse error: {}", e)))?;

        let mut content = String::new();
        let page_count = pdf.get_pages().len();

        for (_, page_id) in pdf.get_pages() {
            if let Ok(page) = pdf.get_object(page_id) {
                if let Ok(page_dict) = page.as_dict() {
                    if let Ok(contents) = page_dict.get(b"Contents") {
                        if let Ok(content_stream) = contents.as_array() {
                            for obj in content_stream {
                                if let lopdf::Object::Reference(obj_id) = obj {
                                    if let Ok(stream) = pdf.get_object(*obj_id) {
                                        if let Ok(stream_dict) = stream.as_stream() {
                                            if let Ok(decoded) = stream_dict.decompressed_content()
                                            {
                                                let text = String::from_utf8_lossy(&decoded);
                                                content.push_str(&text);
                                                content.push('\n');
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(ParsedDocument {
            content,
            metadata: DocumentMetadata {
                title: path.file_name().and_then(|n| n.to_str()).map(String::from),
                page_count: Some(page_count),
                ..Default::default()
            },
        })
    }

    fn parse_word(path: &Path) -> AppResult<ParsedDocument> {
        let _content = std::fs::read(path)?;

        Ok(ParsedDocument {
            content: String::new(),
            metadata: DocumentMetadata {
                title: path.file_name().and_then(|n| n.to_str()).map(String::from),
                ..Default::default()
            },
        })
    }

    fn parse_excel(path: &Path) -> AppResult<ParsedDocument> {
        let _content = std::fs::read(path)?;

        Ok(ParsedDocument {
            content: String::new(),
            metadata: DocumentMetadata {
                title: path.file_name().and_then(|n| n.to_str()).map(String::from),
                ..Default::default()
            },
        })
    }
}
