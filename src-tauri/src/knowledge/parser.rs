use crate::{AppError, AppResult};
use calamine::Reader;
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
        let content_bytes = std::fs::read(path)?;

        let content = pdf_extract::extract_text_from_mem(&content_bytes)
            .map_err(|e| AppError::DocumentParse(format!("PDF parse error: {}", e)))?;

        let pdf = lopdf::Document::load_mem(&content_bytes)
            .map_err(|e| AppError::DocumentParse(format!("PDF load error: {}", e)))?;
        let page_count = pdf.get_pages().len();

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
        let content_bytes = std::fs::read(path)?;

        let docx = docx_rs::read_docx(&content_bytes)
            .map_err(|e| AppError::DocumentParse(format!("Word parse error: {}", e)))?;

        let mut content = String::new();
        for child in docx.document.children.iter() {
            match child {
                docx_rs::DocumentChild::Paragraph(para) => {
                    for p_child in para.children.iter() {
                        if let docx_rs::ParagraphChild::Run(run) = p_child {
                            for r_child in run.children.iter() {
                                if let docx_rs::RunChild::Text(text) = r_child {
                                    content.push_str(&text.text);
                                }
                            }
                        }
                    }
                    content.push('\n');
                }
                docx_rs::DocumentChild::Table(table) => {
                    for table_child in table.rows.iter() {
                        if let docx_rs::TableChild::TableRow(row) = table_child {
                            for row_child in row.cells.iter() {
                                if let docx_rs::TableRowChild::TableCell(cell) = row_child {
                                    for cell_child in cell.children.iter() {
                                        if let docx_rs::TableCellContent::Paragraph(para) =
                                            cell_child
                                        {
                                            for p_child in para.children.iter() {
                                                if let docx_rs::ParagraphChild::Run(run) = p_child {
                                                    for r_child in run.children.iter() {
                                                        if let docx_rs::RunChild::Text(text) =
                                                            r_child
                                                        {
                                                            content.push_str(&text.text);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    content.push('\t');
                                }
                            }
                            content.push('\n');
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(ParsedDocument {
            content,
            metadata: DocumentMetadata {
                title: path.file_name().and_then(|n| n.to_str()).map(String::from),
                ..Default::default()
            },
        })
    }

    fn parse_excel(path: &Path) -> AppResult<ParsedDocument> {
        let mut workbook = calamine::open_workbook_auto(path)
            .map_err(|e| AppError::DocumentParse(format!("Excel parse error: {}", e)))?;

        let mut content = String::new();
        let sheets = workbook.sheet_names().to_vec();

        for sheet_name in sheets {
            content.push_str(&format!("# {}\n\n", sheet_name));

            if let Some(range_result) = workbook.worksheet_range(&sheet_name) {
                let range = range_result
                    .map_err(|e| AppError::DocumentParse(format!("Excel sheet error: {}", e)))?;

                for row in range.rows() {
                    for cell in row.iter() {
                        let cell_text = match cell {
                            calamine::DataType::Int(i) => i.to_string(),
                            calamine::DataType::Float(f) => f.to_string(),
                            calamine::DataType::String(s) => s.clone(),
                            calamine::DataType::Bool(b) => b.to_string(),
                            calamine::DataType::DateTime(dt) => dt.to_string(),
                            _ => String::new(),
                        };
                        content.push_str(&cell_text);
                        content.push('\t');
                    }
                    content.push('\n');
                }
            }
            content.push('\n');
        }

        Ok(ParsedDocument {
            content,
            metadata: DocumentMetadata {
                title: path.file_name().and_then(|n| n.to_str()).map(String::from),
                ..Default::default()
            },
        })
    }
}
