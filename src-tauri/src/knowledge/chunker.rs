use super::DocumentChunk;

pub struct Chunker {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl Default for Chunker {
    fn default() -> Self {
        Self {
            chunk_size: 100,
            chunk_overlap: 20,
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
        let mut chunks = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let total_len = chars.len();

        if total_len == 0 {
            return chunks;
        }

        let mut start = 0;
        let mut index = 0;

        while start < total_len {
            let end = (start + self.chunk_size).min(total_len);
            let chunk_content: String = chars[start..end].iter().collect();

            chunks.push(DocumentChunk::new(
                document_id.to_string(),
                index,
                chunk_content,
            ));

            index += 1;

            if end >= total_len {
                break;
            }

            start = end.saturating_sub(self.chunk_overlap);
        }

        chunks
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
        assert!(chunks[0].content.chars().count() <= 100);
    }

    #[test]
    fn test_chunk_multiple_paragraphs() {
        let content = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = chunk_document("test", content);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.content.chars().count() <= 100);
        }
    }

    #[test]
    fn test_chunk_long_content() {
        let content: String = "a".repeat(1000);
        let chunks = chunk_document("test", &content);
        for chunk in &chunks {
            assert!(chunk.content.chars().count() <= 100);
        }
        assert!(chunks.len() >= 5);
    }
}
