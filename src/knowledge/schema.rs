//! Schema definitions for the knowledge index
//!
//! Defines the Arrow schema for storing command knowledge.

use arrow_array::{
    ArrayRef, Float32Array, RecordBatch, RecordBatchIterator, StringArray, TimestampSecondArray,
};
use arrow_schema::{DataType, Field, Schema, TimeUnit};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Embedding dimension (MiniLM-L6-v2 produces 384-dim vectors)
pub const EMBEDDING_DIM: usize = 384;

/// Type of knowledge entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    /// A successfully executed command
    Success,
    /// A correction made during agentic refinement
    Correction,
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::Success => "success",
            EntryType::Correction => "correction",
        }
    }

    /// Parse entry type from string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "success" => Some(EntryType::Success),
            "correction" => Some(EntryType::Correction),
            _ => None,
        }
    }
}

/// Schema for the knowledge entries table
pub fn knowledge_schema() -> Schema {
    Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("request", DataType::Utf8, false),
        Field::new("command", DataType::Utf8, false),
        Field::new("context", DataType::Utf8, true),
        Field::new(
            "embedding",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                EMBEDDING_DIM as i32,
            ),
            false,
        ),
        Field::new("entry_type", DataType::Utf8, false),
        Field::new(
            "timestamp",
            DataType::Timestamp(TimeUnit::Second, None),
            false,
        ),
        Field::new("original_command", DataType::Utf8, true),
        Field::new("feedback", DataType::Utf8, true),
        Field::new("profile", DataType::Utf8, true),
    ])
}

/// Builder for creating knowledge entry record batches
pub struct EntryBuilder {
    ids: Vec<String>,
    requests: Vec<String>,
    commands: Vec<String>,
    contexts: Vec<Option<String>>,
    embeddings: Vec<Vec<f32>>,
    entry_types: Vec<String>,
    timestamps: Vec<i64>,
    original_commands: Vec<Option<String>>,
    feedbacks: Vec<Option<String>>,
    profiles: Vec<Option<String>>,
}

impl Default for EntryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EntryBuilder {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            requests: Vec::new(),
            commands: Vec::new(),
            contexts: Vec::new(),
            embeddings: Vec::new(),
            entry_types: Vec::new(),
            timestamps: Vec::new(),
            original_commands: Vec::new(),
            feedbacks: Vec::new(),
            profiles: Vec::new(),
        }
    }

    /// Add a successful command entry
    #[allow(clippy::too_many_arguments)]
    pub fn add_success(
        &mut self,
        id: impl Into<String>,
        request: impl Into<String>,
        command: impl Into<String>,
        context: Option<String>,
        embedding: Vec<f32>,
        timestamp: DateTime<Utc>,
        profile: Option<String>,
    ) {
        self.ids.push(id.into());
        self.requests.push(request.into());
        self.commands.push(command.into());
        self.contexts.push(context);
        self.embeddings.push(embedding);
        self.entry_types
            .push(EntryType::Success.as_str().to_string());
        self.timestamps.push(timestamp.timestamp());
        self.original_commands.push(None);
        self.feedbacks.push(None);
        self.profiles.push(profile);
    }

    /// Add a correction entry
    #[allow(clippy::too_many_arguments)]
    pub fn add_correction(
        &mut self,
        id: impl Into<String>,
        request: impl Into<String>,
        corrected_command: impl Into<String>,
        original_command: impl Into<String>,
        feedback: Option<String>,
        embedding: Vec<f32>,
        timestamp: DateTime<Utc>,
        profile: Option<String>,
    ) {
        self.ids.push(id.into());
        self.requests.push(request.into());
        self.commands.push(corrected_command.into());
        self.contexts.push(None);
        self.embeddings.push(embedding);
        self.entry_types
            .push(EntryType::Correction.as_str().to_string());
        self.timestamps.push(timestamp.timestamp());
        self.original_commands.push(Some(original_command.into()));
        self.feedbacks.push(feedback);
        self.profiles.push(profile);
    }

    /// Build the record batch
    pub fn build(self) -> crate::knowledge::Result<RecordBatch> {
        use arrow_array::FixedSizeListArray;

        let schema = Arc::new(knowledge_schema());

        // Build embedding array
        let flat_embeddings: Vec<f32> = self.embeddings.into_iter().flatten().collect();
        let embedding_values = Float32Array::from(flat_embeddings);
        let embedding_field = Arc::new(Field::new("item", DataType::Float32, true));
        let embedding_array = FixedSizeListArray::try_new(
            embedding_field,
            EMBEDDING_DIM as i32,
            Arc::new(embedding_values),
            None,
        )
        .map_err(|e| crate::knowledge::KnowledgeError::Schema(e.to_string()))?;

        let columns: Vec<ArrayRef> = vec![
            Arc::new(StringArray::from(self.ids)),
            Arc::new(StringArray::from(self.requests)),
            Arc::new(StringArray::from(self.commands)),
            Arc::new(StringArray::from(self.contexts)),
            Arc::new(embedding_array),
            Arc::new(StringArray::from(self.entry_types)),
            Arc::new(TimestampSecondArray::from(self.timestamps)),
            Arc::new(StringArray::from(self.original_commands)),
            Arc::new(StringArray::from(self.feedbacks)),
            Arc::new(StringArray::from(self.profiles)),
        ];

        RecordBatch::try_new(schema, columns)
            .map_err(|e| crate::knowledge::KnowledgeError::Schema(e.to_string()))
    }

    /// Build as an iterator for adding to LanceDB
    #[allow(dead_code)]
    pub fn build_iter(
        self,
    ) -> crate::knowledge::Result<
        RecordBatchIterator<
            std::iter::Once<std::result::Result<RecordBatch, arrow_schema::ArrowError>>,
        >,
    > {
        let batch = self.build()?;
        let schema = batch.schema();
        Ok(RecordBatchIterator::new(std::iter::once(Ok(batch)), schema))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_type_conversion() {
        assert_eq!(EntryType::Success.as_str(), "success");
        assert_eq!(EntryType::Correction.as_str(), "correction");
        assert_eq!(EntryType::parse("success"), Some(EntryType::Success));
        assert_eq!(EntryType::parse("correction"), Some(EntryType::Correction));
        assert_eq!(EntryType::parse("unknown"), None);
    }

    #[test]
    fn test_schema_fields() {
        let schema = knowledge_schema();
        assert_eq!(schema.fields().len(), 10);
        assert!(schema.field_with_name("id").is_ok());
        assert!(schema.field_with_name("request").is_ok());
        assert!(schema.field_with_name("command").is_ok());
        assert!(schema.field_with_name("embedding").is_ok());
        assert!(schema.field_with_name("profile").is_ok());
    }

    #[test]
    fn test_entry_builder() {
        let mut builder = EntryBuilder::new();
        builder.add_success(
            "test-id",
            "list files",
            "ls -la",
            Some("rust project".to_string()),
            vec![0.0; EMBEDDING_DIM],
            Utc::now(),
            None,
        );

        let batch = builder.build().unwrap();
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 10);
    }
}
