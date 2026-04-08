use crate::error::Result;
use crate::memory::archive::ConversationArchive;
use crate::models::Message;

/// Semantic search over conversation history.
/// Currently uses FTS5 full-text search as the primary search mechanism.
/// Vector-based semantic search (via fastembed-rs) will be added in a future phase
/// once the embedding pipeline is integrated.
pub struct SemanticSearch<'a> {
    archive: &'a ConversationArchive,
}

impl<'a> SemanticSearch<'a> {
    pub fn new(archive: &'a ConversationArchive) -> Self {
        Self { archive }
    }

    /// Search for relevant past messages using FTS5 full-text search.
    /// This is the fallback/baseline search. Vector similarity search
    /// will be layered on top when fastembed-rs is integrated.
    pub async fn search_relevant(&self, query: &str, limit: usize) -> Result<Vec<Message>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        self.archive.search_messages(query, limit).await
    }

    /// Extract key terms from recent messages to use as search queries.
    /// This provides context-aware retrieval without embeddings.
    pub fn extract_search_terms(messages: &[Message]) -> Vec<String> {
        let mut terms = Vec::new();
        // Use the last few messages to extract potential search terms
        for msg in messages.iter().rev().take(3) {
            // Split into words, filter short words and common stopwords
            for word in msg.content.split_whitespace() {
                let clean = word
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase();
                if clean.len() > 3
                    && !STOPWORDS.contains(&clean.as_str())
                    && !terms.contains(&clean)
                {
                    terms.push(clean);
                }
            }
        }
        terms.truncate(5);
        terms
    }
}

const STOPWORDS: &[&str] = &[
    "this", "that", "with", "from", "have", "been", "were", "they", "will", "would", "could",
    "should", "what", "when", "where", "which", "about", "their", "there", "here", "your", "just",
    "like", "know", "think", "going", "want", "need", "some", "also", "than", "then", "more",
    "very", "much", "into",
];
