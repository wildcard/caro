//! RAG retrieval logic for enhancing command generation

use super::client::KnowledgeBaseClient;
use super::collections::CollectionType;
use super::profiles::UserProfile;
use super::KnowledgeBaseError;
use crate::models::{CommandRequest, ShellType};
use chromadb::v2::collection::ChromaCollection;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::{debug, info};

/// Context for retrieval operations
#[derive(Debug, Clone)]
pub struct RetrievalContext {
    /// User prompt
    pub prompt: String,

    /// User profile
    pub profile: UserProfile,

    /// Current directory (for project context)
    pub current_dir: Option<String>,

    /// Shell type
    pub shell_type: ShellType,

    /// Maximum documents to retrieve
    pub max_docs: usize,

    /// Minimum similarity score
    pub min_similarity: f32,
}

impl RetrievalContext {
    pub fn from_request(request: &CommandRequest, profile: UserProfile) -> Self {
        Self {
            prompt: request.input.clone(),
            profile,
            current_dir: request.context.clone(),
            shell_type: request.shell,
            max_docs: 5,
            min_similarity: 0.7,
        }
    }
}

/// Retrieved document with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedDocument {
    /// Document ID
    pub id: String,

    /// Document content
    pub content: String,

    /// Similarity score (0.0 to 1.0)
    pub score: f32,

    /// Source collection
    pub source: String,

    /// Metadata
    pub metadata: HashMap<String, JsonValue>,
}

/// Result of retrieval operation
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    /// Retrieved documents
    pub documents: Vec<RetrievedDocument>,

    /// Execution history matches
    pub similar_executions: Vec<RetrievedDocument>,

    /// User preferences
    pub preferences: Vec<RetrievedDocument>,

    /// Learned mistakes
    pub mistakes: Vec<RetrievedDocument>,

    /// Project context
    pub project_context: Vec<RetrievedDocument>,
}

impl RetrievalResult {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            similar_executions: Vec::new(),
            preferences: Vec::new(),
            mistakes: Vec::new(),
            project_context: Vec::new(),
        }
    }

    /// Build enhanced prompt with retrieved context
    pub fn build_enhanced_prompt(&self, original_prompt: &str) -> String {
        let mut enhanced = String::new();

        enhanced.push_str("# User Request\n");
        enhanced.push_str(original_prompt);
        enhanced.push_str("\n\n");

        // Add relevant documentation
        if !self.documents.is_empty() {
            enhanced.push_str("# Relevant Documentation\n");
            for (idx, doc) in self.documents.iter().take(3).enumerate() {
                enhanced.push_str(&format!("\n## Reference {}\n", idx + 1));
                enhanced.push_str(&self.truncate_content(&doc.content, 500));
                enhanced.push_str("\n");
            }
            enhanced.push_str("\n");
        }

        // Add similar past executions
        if !self.similar_executions.is_empty() {
            enhanced.push_str("# Similar Past Commands\n");
            for doc in self.similar_executions.iter().take(3) {
                if let Some(cmd) = doc.metadata.get("command") {
                    enhanced.push_str(&format!("- {}\n", cmd.as_str().unwrap_or("")));
                }
            }
            enhanced.push_str("\n");
        }

        // Add user preferences
        if !self.preferences.is_empty() {
            enhanced.push_str("# User Preferences\n");
            for doc in self.preferences.iter().take(2) {
                if let Some(pattern) = doc.metadata.get("command_pattern") {
                    enhanced.push_str(&format!("- Prefers: {}\n", pattern.as_str().unwrap_or("")));
                }
            }
            enhanced.push_str("\n");
        }

        // Add learned mistakes
        if !self.mistakes.is_empty() {
            enhanced.push_str("# Previous Mistakes to Avoid\n");
            for doc in self.mistakes.iter().take(2) {
                if let Some(error) = doc.metadata.get("error_type") {
                    enhanced.push_str(&format!("- Avoid: {}\n", error.as_str().unwrap_or("")));
                }
                if let Some(correction) = doc.metadata.get("correction") {
                    enhanced.push_str(&format!("  Instead: {}\n", correction.as_str().unwrap_or("")));
                }
            }
            enhanced.push_str("\n");
        }

        // Add project context
        if !self.project_context.is_empty() {
            enhanced.push_str("# Project-Specific Context\n");
            for doc in self.project_context.iter().take(2) {
                enhanced.push_str(&self.truncate_content(&doc.content, 300));
                enhanced.push_str("\n");
            }
            enhanced.push_str("\n");
        }

        enhanced.push_str("# Instructions\n");
        enhanced.push_str("Generate a shell command based on the user request above, taking into account the relevant documentation, past commands, user preferences, and any mistakes to avoid.\n");

        enhanced
    }

    /// Truncate content to maximum length
    fn truncate_content(&self, content: &str, max_len: usize) -> String {
        if content.len() <= max_len {
            content.to_string()
        } else {
            format!("{}...", &content[..max_len])
        }
    }

    /// Get total number of retrieved documents
    pub fn total_documents(&self) -> usize {
        self.documents.len()
            + self.similar_executions.len()
            + self.preferences.len()
            + self.mistakes.len()
            + self.project_context.len()
    }
}

impl Default for RetrievalResult {
    fn default() -> Self {
        Self::new()
    }
}

/// RAG retriever for enhancing prompts
pub struct RAGRetriever {
    client: KnowledgeBaseClient,
}

impl RAGRetriever {
    pub fn new(client: KnowledgeBaseClient) -> Self {
        Self { client }
    }

    /// Retrieve relevant context for a command request
    pub async fn retrieve(
        &self,
        context: &RetrievalContext,
    ) -> Result<RetrievalResult, KnowledgeBaseError> {
        info!("Retrieving context for prompt: {}", context.prompt);
        let mut result = RetrievalResult::new();

        // Retrieve from each collection type
        result.documents = self
            .retrieve_from_collection(
                CollectionType::CommandDocs,
                &context.prompt,
                context.max_docs,
            )
            .await?;

        result.similar_executions = self
            .retrieve_execution_history(context)
            .await?;

        result.preferences = self
            .retrieve_user_preferences(context)
            .await?;

        result.mistakes = self
            .retrieve_learned_mistakes(context)
            .await?;

        if let Some(project_dir) = &context.current_dir {
            result.project_context = self
                .retrieve_project_context(project_dir, &context.prompt, context.max_docs)
                .await?;
        }

        info!("Retrieved {} total documents", result.total_documents());

        Ok(result)
    }

    /// Retrieve documents from a specific collection
    async fn retrieve_from_collection(
        &self,
        collection_type: CollectionType,
        query: &str,
        n_results: usize,
    ) -> Result<Vec<RetrievedDocument>, KnowledgeBaseError> {
        let collection = self
            .client
            .get_or_create_collection(collection_type.collection_name())
            .await?;

        let query_result = self
            .client
            .query_collection(&collection, vec![query.to_string()], Some(n_results), None)
            .await?;

        // Convert QueryResult to RetrievedDocument
        let mut documents = Vec::new();

        if let Some(ids) = query_result.ids {
            for (idx, id_list) in ids.iter().enumerate() {
                for (doc_idx, id) in id_list.iter().enumerate() {
                    let content = query_result
                        .documents
                        .as_ref()
                        .and_then(|docs| docs.get(idx))
                        .and_then(|doc_list| doc_list.get(doc_idx))
                        .cloned()
                        .unwrap_or_default();

                    let distance = query_result
                        .distances
                        .as_ref()
                        .and_then(|dists| dists.get(idx))
                        .and_then(|dist_list| dist_list.get(doc_idx))
                        .copied()
                        .unwrap_or(1.0);

                    // Convert distance to similarity score (assuming cosine distance)
                    let score = 1.0 - distance;

                    let metadata = query_result
                        .metadatas
                        .as_ref()
                        .and_then(|metas| metas.get(idx))
                        .and_then(|meta_list| meta_list.get(doc_idx))
                        .cloned()
                        .unwrap_or_default();

                    documents.push(RetrievedDocument {
                        id: id.clone(),
                        content,
                        score,
                        source: collection_type.to_string(),
                        metadata,
                    });
                }
            }
        }

        debug!(
            "Retrieved {} documents from {}",
            documents.len(),
            collection_type
        );

        Ok(documents)
    }

    /// Retrieve similar execution history
    async fn retrieve_execution_history(
        &self,
        context: &RetrievalContext,
    ) -> Result<Vec<RetrievedDocument>, KnowledgeBaseError> {
        let mut where_clause = HashMap::new();
        where_clause.insert(
            "user_profile".to_string(),
            JsonValue::String(context.profile.name.clone()),
        );
        where_clause.insert("success".to_string(), JsonValue::Bool(true));

        let collection = self
            .client
            .get_or_create_collection(CollectionType::ExecutionHistory.collection_name())
            .await?;

        let query_result = self
            .client
            .query_collection(
                &collection,
                vec![context.prompt.clone()],
                Some(context.max_docs),
                Some(where_clause),
            )
            .await?;

        self.convert_query_result(query_result, CollectionType::ExecutionHistory)
    }

    /// Retrieve user preferences
    async fn retrieve_user_preferences(
        &self,
        context: &RetrievalContext,
    ) -> Result<Vec<RetrievedDocument>, KnowledgeBaseError> {
        let mut where_clause = HashMap::new();
        where_clause.insert(
            "user_profile".to_string(),
            JsonValue::String(context.profile.name.clone()),
        );

        let collection = self
            .client
            .get_or_create_collection(CollectionType::UserPreferences.collection_name())
            .await?;

        let query_result = self
            .client
            .query_collection(
                &collection,
                vec![context.prompt.clone()],
                Some(context.max_docs),
                Some(where_clause),
            )
            .await?;

        self.convert_query_result(query_result, CollectionType::UserPreferences)
    }

    /// Retrieve learned mistakes
    async fn retrieve_learned_mistakes(
        &self,
        context: &RetrievalContext,
    ) -> Result<Vec<RetrievedDocument>, KnowledgeBaseError> {
        let mut where_clause = HashMap::new();
        where_clause.insert(
            "user_profile".to_string(),
            JsonValue::String(context.profile.name.clone()),
        );

        let collection = self
            .client
            .get_or_create_collection(CollectionType::MistakesLearned.collection_name())
            .await?;

        let query_result = self
            .client
            .query_collection(
                &collection,
                vec![context.prompt.clone()],
                Some(context.max_docs),
                Some(where_clause),
            )
            .await?;

        self.convert_query_result(query_result, CollectionType::MistakesLearned)
    }

    /// Retrieve project-specific context
    async fn retrieve_project_context(
        &self,
        project_dir: &str,
        query: &str,
        n_results: usize,
    ) -> Result<Vec<RetrievedDocument>, KnowledgeBaseError> {
        let mut where_clause = HashMap::new();
        where_clause.insert("project_path".to_string(), JsonValue::String(project_dir.to_string()));

        let collection = self
            .client
            .get_or_create_collection(CollectionType::ProjectContext.collection_name())
            .await?;

        let query_result = self
            .client
            .query_collection(&collection, vec![query.to_string()], Some(n_results), Some(where_clause))
            .await?;

        self.convert_query_result(query_result, CollectionType::ProjectContext)
    }

    /// Convert ChromaDB QueryResult to RetrievedDocument
    fn convert_query_result(
        &self,
        query_result: chromadb::v2::collection::QueryResult,
        collection_type: CollectionType,
    ) -> Result<Vec<RetrievedDocument>, KnowledgeBaseError> {
        let mut documents = Vec::new();

        if let Some(ids) = query_result.ids {
            for (idx, id_list) in ids.iter().enumerate() {
                for (doc_idx, id) in id_list.iter().enumerate() {
                    let content = query_result
                        .documents
                        .as_ref()
                        .and_then(|docs| docs.get(idx))
                        .and_then(|doc_list| doc_list.get(doc_idx))
                        .cloned()
                        .unwrap_or_default();

                    let distance = query_result
                        .distances
                        .as_ref()
                        .and_then(|dists| dists.get(idx))
                        .and_then(|dist_list| dist_list.get(doc_idx))
                        .copied()
                        .unwrap_or(1.0);

                    let score = 1.0 - distance;

                    let metadata = query_result
                        .metadatas
                        .as_ref()
                        .and_then(|metas| metas.get(idx))
                        .and_then(|meta_list| meta_list.get(doc_idx))
                        .cloned()
                        .unwrap_or_default();

                    documents.push(RetrievedDocument {
                        id: id.clone(),
                        content,
                        score,
                        source: collection_type.to_string(),
                        metadata,
                    });
                }
            }
        }

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SafetyLevel;

    #[test]
    fn test_retrieval_result_enhanced_prompt() {
        let mut result = RetrievalResult::new();

        result.documents.push(RetrievedDocument {
            id: "doc1".to_string(),
            content: "find is a command that searches for files".to_string(),
            score: 0.9,
            source: "command_docs".to_string(),
            metadata: HashMap::new(),
        });

        let enhanced = result.build_enhanced_prompt("find all log files");

        assert!(enhanced.contains("User Request"));
        assert!(enhanced.contains("find all log files"));
        assert!(enhanced.contains("Relevant Documentation"));
        assert!(enhanced.contains("find is a command"));
    }

    #[test]
    fn test_retrieval_context() {
        let profile = UserProfile::new("test", "Test profile", ShellType::Bash, SafetyLevel::Moderate);

        let request = CommandRequest::new("list files", ShellType::Bash);

        let context = RetrievalContext::from_request(&request, profile);

        assert_eq!(context.prompt, "list files");
        assert_eq!(context.shell_type, ShellType::Bash);
    }
}
