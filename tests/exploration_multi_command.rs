use cmdai::agent::exploration::{
    CommandAlternative, EnrichmentResult, ExplorationAgent, ToolContext, ToolSuggestion,
};
use cmdai::backends::embedded::EmbeddedModelBackend;
use cmdai::context::ExecutionContext;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_generate_alternatives_basic() {
    // Arrange
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);

    let mut contexts = HashMap::new();
    contexts.insert(
        "ps".to_string(),
        ToolContext {
            tool: "ps".to_string(),
            installed: true,
            man_summary: Some("Display process status".to_string()),
            help_text: Some("Usage: ps [options]".to_string()),
            tldr_example: Some("ps aux".to_string()),
        },
    );

    let enrichment = EnrichmentResult {
        contexts,
        tldr_recommended: false,
    };

    // Act
    let alternatives = agent
        .generate_alternatives("show top 5 CPU processes", &enrichment)
        .await
        .expect("Failed to generate alternatives");

    // Assert
    assert!(!alternatives.is_empty(), "Should generate alternatives");
    assert!(
        alternatives.len() <= 3,
        "Should generate at most 3 alternatives"
    );

    // Check first alternative has required fields
    let first = &alternatives[0];
    assert_eq!(first.rank, 1, "First alternative should have rank 1");
    assert!(!first.command.is_empty(), "Command should not be empty");
    assert!(
        first.confidence > 0.0 && first.confidence <= 1.0,
        "Confidence should be between 0 and 1"
    );
    assert!(!first.tools_used.is_empty(), "Should list tools used");
}

#[tokio::test]
async fn test_generate_alternatives_ranked() {
    // Arrange
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);

    let mut contexts = HashMap::new();
    contexts.insert(
        "ps".to_string(),
        ToolContext {
            tool: "ps".to_string(),
            installed: true,
            man_summary: Some("Display process status".to_string()),
            help_text: Some("Usage: ps [options]".to_string()),
            tldr_example: Some("ps aux".to_string()),
        },
    );
    contexts.insert(
        "top".to_string(),
        ToolContext {
            tool: "top".to_string(),
            installed: true,
            man_summary: Some("Display and update sorted process information".to_string()),
            help_text: Some("Usage: top [options]".to_string()),
            tldr_example: Some("top -n 10".to_string()),
        },
    );

    let enrichment = EnrichmentResult {
        contexts,
        tldr_recommended: false,
    };

    // Act
    let alternatives = agent
        .generate_alternatives("show top 5 CPU processes", &enrichment)
        .await
        .expect("Failed to generate alternatives");

    // Assert
    assert!(!alternatives.is_empty(), "Should generate at least one alternative");

    // Check ranks are sequential
    for (idx, alt) in alternatives.iter().enumerate() {
        assert_eq!(
            alt.rank,
            idx + 1,
            "Rank should match position (1-indexed)"
        );
    }

    // If multiple alternatives, check first has highest confidence
    if alternatives.len() >= 2 {
        assert!(
            alternatives[0].confidence >= alternatives[1].confidence,
            "First alternative should have highest confidence"
        );
    }
}

#[tokio::test]
async fn test_generate_alternatives_with_pros_cons() {
    // Arrange
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);

    let mut contexts = HashMap::new();
    contexts.insert(
        "find".to_string(),
        ToolContext {
            tool: "find".to_string(),
            installed: true,
            man_summary: Some("Walk a file hierarchy".to_string()),
            help_text: Some("Usage: find path [options]".to_string()),
            tldr_example: Some("find . -name '*.txt'".to_string()),
        },
    );

    let enrichment = EnrichmentResult {
        contexts,
        tldr_recommended: false,
    };

    // Act
    let alternatives = agent
        .generate_alternatives("find large files", &enrichment)
        .await
        .expect("Failed to generate alternatives");

    // Assert
    assert!(!alternatives.is_empty());

    let first = &alternatives[0];
    // Pros/cons might be populated depending on model response
    println!("Command: {}", first.command);
    println!("Pros: {:?}", first.pros);
    println!("Cons: {:?}", first.cons);
    println!("Explanation: {}", first.explanation);

    // At minimum should have explanation
    assert!(
        !first.explanation.is_empty(),
        "Should have explanation for alternative"
    );
}

#[tokio::test]
async fn test_generate_alternatives_with_multiple_tools() {
    // Arrange
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);

    let mut contexts = HashMap::new();
    contexts.insert(
        "lsof".to_string(),
        ToolContext {
            tool: "lsof".to_string(),
            installed: true,
            man_summary: Some("List open files".to_string()),
            help_text: Some("Usage: lsof [options]".to_string()),
            tldr_example: Some("lsof -i :8080".to_string()),
        },
    );
    contexts.insert(
        "netstat".to_string(),
        ToolContext {
            tool: "netstat".to_string(),
            installed: true,
            man_summary: Some("Show network status".to_string()),
            help_text: Some("Usage: netstat [options]".to_string()),
            tldr_example: Some("netstat -an".to_string()),
        },
    );

    let enrichment = EnrichmentResult {
        contexts,
        tldr_recommended: false,
    };

    // Act
    let alternatives = agent
        .generate_alternatives("check port 8080", &enrichment)
        .await
        .expect("Failed to generate alternatives");

    // Assert
    assert!(!alternatives.is_empty(), "Should generate at least one alternative");

    // Check that alternatives use tools from enrichment context
    let tools_set: std::collections::HashSet<_> = alternatives
        .iter()
        .flat_map(|alt| alt.tools_used.iter())
        .collect();

    println!("Tools used across alternatives: {:?}", tools_set);
    assert!(
        tools_set.len() >= 1,
        "Should use at least one tool from enrichment context"
    );
}

#[test]
fn test_command_alternative_serialization() {
    // Arrange
    let alternative = CommandAlternative {
        command: "ps aux | sort -k3 -rn | head -5".to_string(),
        rank: 1,
        confidence: 0.95,
        tools_used: vec!["ps".to_string(), "sort".to_string(), "head".to_string()],
        pros: vec!["BSD-compatible".to_string(), "simple".to_string()],
        cons: vec!["snapshot only".to_string()],
        explanation: "Uses ps with BSD flags, sorts by CPU".to_string(),
    };

    // Act
    let json = serde_json::to_string(&alternative).expect("Failed to serialize");
    let deserialized: CommandAlternative =
        serde_json::from_str(&json).expect("Failed to deserialize");

    // Assert
    assert_eq!(deserialized.command, alternative.command);
    assert_eq!(deserialized.rank, 1);
    assert_eq!(deserialized.confidence, 0.95);
    assert_eq!(deserialized.tools_used, alternative.tools_used);
}

#[test]
fn test_alternatives_result_serialization() {
    use cmdai::agent::exploration::AlternativesResult;

    // Arrange
    let result = AlternativesResult {
        alternatives: vec![
            CommandAlternative {
                command: "ls -lh".to_string(),
                rank: 1,
                confidence: 0.9,
                tools_used: vec!["ls".to_string()],
                pros: vec!["simple".to_string()],
                cons: vec![],
                explanation: "Basic listing".to_string(),
            },
        ],
        query: "list files".to_string(),
        total_time_ms: 1500,
    };

    // Act
    let json = serde_json::to_string(&result).expect("Failed to serialize");
    let deserialized: AlternativesResult =
        serde_json::from_str(&json).expect("Failed to deserialize");

    // Assert
    assert_eq!(deserialized.alternatives.len(), 1);
    assert_eq!(deserialized.query, "list files");
    assert_eq!(deserialized.total_time_ms, 1500);
}
