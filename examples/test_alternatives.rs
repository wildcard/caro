use cmdai::agent::exploration::{ExplorationAgent, ToolContext, EnrichmentResult};
use cmdai::backends::embedded::EmbeddedModelBackend;
use cmdai::context::ExecutionContext;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Setup
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

    // Test
    println!("Testing alternative generation...\n");
    match agent.generate_alternatives("show top 5 CPU processes", &enrichment).await {
        Ok(alternatives) => {
            println!("✅ Got {} alternatives:", alternatives.len());
            for alt in &alternatives {
                println!("\n━━━ Alternative #{} ━━━", alt.rank);
                println!("Command: {}", alt.command);
                println!("Confidence: {:.2}", alt.confidence);
                println!("Tools: {:?}", alt.tools_used);
                println!("Pros: {:?}", alt.pros);
                println!("Cons: {:?}", alt.cons);
                println!("Explanation: {}", alt.explanation);
            }
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }
}
