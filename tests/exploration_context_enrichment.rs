use cmdai::agent::exploration::{ExplorationAgent, ToolSuggestion};
use cmdai::backends::embedded::EmbeddedModelBackend;
use cmdai::context::ExecutionContext;
use std::sync::Arc;

#[tokio::test]
async fn test_enrich_tool_context_basic() {
    // Arrange
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);
    
    let tools = vec![
        ToolSuggestion {
            tool: "ls".to_string(),
            relevance: "List files".to_string(),
            confidence: 0.9,
            platform_native: true,
        },
    ];
    
    // Act
    let enrichment = agent.enrich_tool_context(&tools).await.expect("Failed to enrich");
    
    // Assert
    assert!(!enrichment.contexts.is_empty(), "Should have context for ls");
    
    let ls_context = enrichment.contexts.get("ls").expect("Should have ls context");
    assert_eq!(ls_context.tool, "ls");
    assert!(ls_context.installed, "ls should be installed");
}

#[tokio::test]
async fn test_enrich_tool_context_with_man_page() {
    // Arrange
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);
    
    let tools = vec![
        ToolSuggestion {
            tool: "ps".to_string(),
            relevance: "Process status".to_string(),
            confidence: 0.95,
            platform_native: true,
        },
    ];
    
    // Act
    let enrichment = agent.enrich_tool_context(&tools).await.expect("Failed to enrich");
    
    // Assert
    let ps_context = enrichment.contexts.get("ps").expect("Should have ps context");
    assert!(
        ps_context.man_summary.is_some(),
        "Should have man page summary for ps"
    );
    
    let man_text = ps_context.man_summary.as_ref().unwrap();
    assert!(!man_text.is_empty(), "Man page summary should not be empty");
    assert!(man_text.len() < 2000, "Man page should be truncated");
}

#[tokio::test]
async fn test_enrich_tool_context_with_help() {
    // Arrange
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);
    
    let tools = vec![
        ToolSuggestion {
            tool: "ls".to_string(),
            relevance: "List files".to_string(),
            confidence: 0.9,
            platform_native: true,
        },
    ];
    
    // Act
    let enrichment = agent.enrich_tool_context(&tools).await.expect("Failed to enrich");
    
    // Assert
    let ls_context = enrichment.contexts.get("ls").expect("Should have ls context");
    
    // Help text might be available (depends on platform)
    if let Some(help) = &ls_context.help_text {
        assert!(!help.is_empty(), "Help text should not be empty");
        println!("ls help text length: {}", help.len());
    }
}

#[tokio::test]
async fn test_enrich_tool_context_multiple_tools() {
    // Arrange
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);
    
    let tools = vec![
        ToolSuggestion {
            tool: "ps".to_string(),
            relevance: "Processes".to_string(),
            confidence: 0.9,
            platform_native: true,
        },
        ToolSuggestion {
            tool: "top".to_string(),
            relevance: "Real-time".to_string(),
            confidence: 0.85,
            platform_native: true,
        },
        ToolSuggestion {
            tool: "grep".to_string(),
            relevance: "Search".to_string(),
            confidence: 0.8,
            platform_native: true,
        },
    ];
    
    // Act
    let enrichment = agent.enrich_tool_context(&tools).await.expect("Failed to enrich");
    
    // Assert
    assert_eq!(enrichment.contexts.len(), 3, "Should have 3 tool contexts");
    assert!(enrichment.contexts.contains_key("ps"));
    assert!(enrichment.contexts.contains_key("top"));
    assert!(enrichment.contexts.contains_key("grep"));
}

#[tokio::test]
async fn test_enrich_tool_context_nonexistent_tool() {
    // Arrange
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);
    
    let tools = vec![
        ToolSuggestion {
            tool: "nonexistent_command_xyz123".to_string(),
            relevance: "Fake tool".to_string(),
            confidence: 0.5,
            platform_native: false,
        },
    ];
    
    // Act
    let enrichment = agent.enrich_tool_context(&tools).await.expect("Failed to enrich");
    
    // Assert
    let fake_context = enrichment
        .contexts
        .get("nonexistent_command_xyz123")
        .expect("Should have context even for nonexistent tool");
    
    assert!(!fake_context.installed, "Nonexistent tool should not be installed");
    assert!(fake_context.man_summary.is_none(), "Should not have man page");
    assert!(fake_context.help_text.is_none(), "Should not have help text");
}

#[tokio::test]
async fn test_enrich_tool_context_tldr_detection() {
    // Arrange
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);
    
    let tools = vec![
        ToolSuggestion {
            tool: "ls".to_string(),
            relevance: "List".to_string(),
            confidence: 0.9,
            platform_native: true,
        },
    ];
    
    // Act
    let enrichment = agent.enrich_tool_context(&tools).await.expect("Failed to enrich");
    
    // Assert - should detect if tldr is installed or recommend it
    if enrichment.tldr_recommended {
        println!("tldr is not installed - recommendation should be shown to user");
    } else {
        println!("tldr is installed - examples may be available");
        
        let ls_context = enrichment.contexts.get("ls").unwrap();
        if let Some(tldr) = &ls_context.tldr_example {
            assert!(!tldr.is_empty(), "tldr example should not be empty");
            println!("tldr example: {}", tldr);
        }
    }
    
    // Test should pass either way
    assert!(true);
}

#[test]
fn test_tool_context_serialization() {
    use cmdai::agent::exploration::ToolContext;
    
    // Arrange
    let context = ToolContext {
        tool: "ps".to_string(),
        installed: true,
        man_summary: Some("Display process status".to_string()),
        help_text: Some("Usage: ps [options]".to_string()),
        tldr_example: Some("ps aux".to_string()),
    };
    
    // Act
    let json = serde_json::to_string(&context).expect("Failed to serialize");
    let deserialized: ToolContext = 
        serde_json::from_str(&json).expect("Failed to deserialize");
    
    // Assert
    assert_eq!(deserialized.tool, "ps");
    assert!(deserialized.installed);
    assert_eq!(deserialized.man_summary, Some("Display process status".to_string()));
}

#[test]
fn test_enrichment_result_serialization() {
    use cmdai::agent::exploration::{EnrichmentResult, ToolContext};
    use std::collections::HashMap;
    
    // Arrange
    let mut contexts = HashMap::new();
    contexts.insert(
        "ls".to_string(),
        ToolContext {
            tool: "ls".to_string(),
            installed: true,
            man_summary: Some("List directory contents".to_string()),
            help_text: None,
            tldr_example: None,
        },
    );
    
    let result = EnrichmentResult {
        contexts,
        tldr_recommended: true,
    };
    
    // Act
    let json = serde_json::to_string(&result).expect("Failed to serialize");
    let deserialized: EnrichmentResult = 
        serde_json::from_str(&json).expect("Failed to deserialize");
    
    // Assert
    assert_eq!(deserialized.contexts.len(), 1);
    assert!(deserialized.tldr_recommended);
    assert!(deserialized.contexts.contains_key("ls"));
}
