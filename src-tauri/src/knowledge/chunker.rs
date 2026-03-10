use super::DocumentChunk;

pub struct Chunker {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl Default for Chunker {
    fn default() -> Self {
        Self {
            chunk_size: 500,
            chunk_overlap: 50,
        }
    }
}

impl Chunker {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }

    pub fn chunk(&self, document_id: &str, content: &str) -> Vec<DocumentChunk> {
        let paragraphs = self.split_into_paragraphs(content);
        let chunks = self.merge_paragraphs(paragraphs);

        chunks
            .into_iter()
            .enumerate()
            .map(|(index, chunk_content)| {
                DocumentChunk::new(document_id.to_string(), index, chunk_content)
            })
            .collect()
    }

    fn split_into_paragraphs(&self, content: &str) -> Vec<String> {
        content
            .split('\n')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect()
    }

    fn merge_paragraphs(&self, paragraphs: Vec<String>) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_size = 0;

        for paragraph in paragraphs {
            let paragraph_size = paragraph.chars().count();

            if current_size + paragraph_size > self.chunk_size && !current_chunk.is_empty() {
                chunks.push(current_chunk.trim().to_string());

                let overlap_text = self.get_overlap_text(&current_chunk);
                current_chunk = overlap_text;
                current_size = current_chunk.chars().count();
            }

            if !current_chunk.is_empty() {
                current_chunk.push('\n');
                current_size += 1;
            }
            current_chunk.push_str(&paragraph);
            current_size += paragraph_size;
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        chunks
    }

    fn get_overlap_text(&self, text: &str) -> String {
        if self.chunk_overlap == 0 {
            return String::new();
        }

        let chars: Vec<char> = text.chars().collect();
        let start = chars.len().saturating_sub(self.chunk_overlap);
        chars[start..].iter().collect()
    }
}

pub fn chunk_document(document_id: &str, content: &str) -> Vec<DocumentChunk> {
    Chunker::default().chunk(document_id, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_empty_content() {
        let chunks = chunk_document("test", "");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_single_paragraph() {
        let content = "This is a test paragraph.";
        let chunks = chunk_document("test", content);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, content);
    }

    #[test]
    fn test_chunk_multiple_paragraphs() {
        let content = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = chunk_document("test", content);
        assert!(!chunks.is_empty());
    }
}
