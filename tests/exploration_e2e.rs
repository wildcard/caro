use cmdai::agent::exploration::ExplorationAgent;
use cmdai::backends::embedded::EmbeddedModelBackend;
use cmdai::context::ExecutionContext;
use std::sync::Arc;
use std::time::Instant;

/// End-to-end test: Full exploration pipeline
#[tokio::test]
async fn test_full_exploration_pipeline() {
    // Setup
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);

    let query = "show top 5 CPU-hungry processes";
    let start = Instant::now();

    println!("\n=== Full Exploration Pipeline E2E Test ===");
    println!("Query: {}\n", query);

    // Phase 0: Complexity Assessment
    println!("Phase 0: Assessing complexity...");
    let phase0_start = Instant::now();
    let assessment = agent
        .assess_complexity(query)
        .await
        .expect("Phase 0 failed");
    println!("  ✓ Complexity: {}", if assessment.is_complex { "COMPLEX" } else { "SIMPLE" });
    println!("  ✓ Confidence: {:.2}", assessment.confidence);
    println!("  ✓ Time: {:?}", phase0_start.elapsed());

    // Phase 1: Tool Discovery
    println!("\nPhase 1: Discovering tools...");
    let phase1_start = Instant::now();
    let tools = agent
        .discover_tools(query, false)
        .await
        .expect("Phase 1 failed");
    println!("  ✓ Found {} tools", tools.len());
    for tool in &tools {
        println!("    - {} (confidence: {:.2})", tool.tool, tool.confidence);
    }
    println!("  ✓ Time: {:?}", phase1_start.elapsed());

    // Phase 2: Context Enrichment
    println!("\nPhase 2: Enriching tool context...");
    let phase2_start = Instant::now();
    let enrichment = agent
        .enrich_tool_context(&tools)
        .await
        .expect("Phase 2 failed");
    println!("  ✓ Enriched {} tools", enrichment.contexts.len());
    for (name, ctx) in &enrichment.contexts {
        println!("    - {}: installed={}, has_man={}, has_help={}", 
            name, 
            ctx.installed,
            ctx.man_summary.is_some(),
            ctx.help_text.is_some()
        );
    }
    println!("  ✓ Time: {:?}", phase2_start.elapsed());

    // Phase 3: Command Generation
    println!("\nPhase 3: Generating alternatives...");
    let phase3_start = Instant::now();
    let alternatives = agent
        .generate_alternatives(query, &enrichment)
        .await
        .expect("Phase 3 failed");
    println!("  ✓ Generated {} alternatives", alternatives.len());
    for alt in &alternatives {
        println!("    {}. {} (confidence: {:.2})", 
            alt.rank, 
            alt.command,
            alt.confidence
        );
        println!("       Tools: {:?}", alt.tools_used);
    }
    println!("  ✓ Time: {:?}", phase3_start.elapsed());

    // Summary
    let total_time = start.elapsed();
    println!("\n=== Pipeline Complete ===");
    println!("Total time: {:?}", total_time);
    println!("Performance: {}", if total_time.as_secs() < 10 { "✓ PASS" } else { "✗ SLOW" });

    // Assertions
    assert!(!tools.is_empty(), "Should discover tools");
    assert!(!enrichment.contexts.is_empty(), "Should enrich context");
    assert!(!alternatives.is_empty(), "Should generate alternatives");
    assert!(total_time.as_secs() < 15, "Should complete within 15s");

    // Check that at least one alternative uses discovered tools
    let has_tools = alternatives.iter().any(|alt| !alt.tools_used.is_empty());
    assert!(has_tools, "At least one alternative should use tools");
}

/// Test exploration with file-related query
#[tokio::test]
async fn test_exploration_with_file_context() {
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);

    let query = "find large files over 100MB";

    println!("\n=== File Context Test ===");
    println!("Query: {}\n", query);

    // Check file relevance
    let is_file_related = agent
        .should_include_files(query, cmdai::agent::ExploreFiles::Auto)
        .await
        .expect("File detection failed");

    println!("File related: {}", is_file_related);
    assert!(is_file_related, "Should detect file-related query");

    // Run discovery with file context
    let tools = agent
        .discover_tools(query, is_file_related)
        .await
        .expect("Tool discovery failed");

    println!("Discovered tools: {:?}", tools.iter().map(|t| &t.tool).collect::<Vec<_>>());
    
    // Should find file-related tools
    let has_find = tools.iter().any(|t| t.tool == "find");
    let has_du = tools.iter().any(|t| t.tool == "du");
    
    assert!(
        has_find || has_du,
        "Should discover file-related tools (find or du)"
    );
}

/// Test exploration performance target
#[tokio::test]
async fn test_exploration_performance() {
    let context = ExecutionContext::detect();
    let backend = EmbeddedModelBackend::new().expect("Failed to create backend");
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    let agent = ExplorationAgent::new(backend_arc, context);

    let queries = vec![
        "list files",
        "current directory",
        "show processes",
    ];

    for (idx, query) in queries.iter().enumerate() {
        let start = Instant::now();
        
        let _assessment = agent.assess_complexity(query).await.expect("Failed");
        let elapsed = start.elapsed();
        
        println!("Query {}: '{}' - {}ms", idx + 1, query, elapsed.as_millis());
        
        // First query includes cold start, subsequent should be faster
        let max_time = if idx == 0 { 8 } else { 3 };
        assert!(
            elapsed.as_secs() < max_time,
            "Query {} should be < {}s (was {}s)", idx + 1, max_time, elapsed.as_secs()
        );
    }
}
