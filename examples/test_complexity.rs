// Test program for complexity assessment
use cmdai::agent::exploration::ExplorationAgent;
use cmdai::backends::embedded::EmbeddedModelBackend;
use cmdai::context::ExecutionContext;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("🧪 Testing Complexity Assessment\n");
    
    // Create context
    let context = ExecutionContext::detect();
    println!("Platform: {} {}", context.os, context.arch);
    println!("Shell: {}", context.shell);
    println!("CWD: {}\n", context.cwd.display());
    
    // Create backend
    let backend = EmbeddedModelBackend::new()?;
    let backend_arc: Arc<dyn cmdai::backends::CommandGenerator> = Arc::new(backend);
    
    // Create exploration agent
    let agent = ExplorationAgent::new(backend_arc, context);
    
    // Test queries
    let test_queries = vec![
        ("list files", "Expected: SIMPLE"),
        ("current directory", "Expected: SIMPLE"),
        ("what time is it", "Expected: SIMPLE"),
        ("show top 5 CPU processes", "Expected: COMPLEX"),
        ("find listening TCP ports", "Expected: COMPLEX"),
        ("disk usage sorted by size", "Expected: COMPLEX"),
        ("git commits from last 2 weeks", "Expected: COMPLEX"),
    ];
    
    for (query, expected) in test_queries {
        println!("─────────────────────────────────────────");
        println!("Query: \"{}\"", query);
        println!("{}", expected);
        println!();
        
        match agent.assess_complexity(query).await {
            Ok(assessment) => {
                println!("Complexity: {}", if assessment.is_complex { "COMPLEX ⚡" } else { "SIMPLE ✓" });
                println!("Confidence: {:.1}%", assessment.confidence * 100.0);
                println!("Reasoning: {}", assessment.reasoning);
                
                if !assessment.likely_tools.is_empty() {
                    println!("Tools: {}", assessment.likely_tools.join(", "));
                }
                
                if let Some(cmd) = &assessment.quick_command {
                    println!("Quick command: {}", cmd);
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
        
        println!();
    }
    
    println!("─────────────────────────────────────────");
    println!("\n✅ Complexity assessment test complete!");
    
    Ok(())
}
