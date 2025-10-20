// Debug test to understand backend selection behavior
// This test will help us see which backends are being selected and why

use cmdai::{
    cli::{CliApp, CliConfig, OutputFormat},
    models::{SafetyLevel, ShellType},
    backends::{CommandGenerator, embedded::EmbeddedModelBackend, decision_tree::DecisionTreeBackend},
};
use std::sync::Arc;

#[tokio::test]
async fn debug_backend_selection_priority() {
    println!("=== Backend Selection Debug Test ===");
    
    // Test 1: Direct embedded backend test
    println!("\n1. Testing Embedded Backend Directly:");
    println!("-------------------------------------");
    
    match EmbeddedModelBackend::new() {
        Ok(embedded_backend) => {
            println!("✅ Embedded backend created successfully");
            
            // Test if it's available
            let is_available = embedded_backend.is_available().await;
            println!("   Available: {}", is_available);
            
            // Get backend info
            let info = embedded_backend.backend_info();
            println!("   Backend type: {:?}", info.backend_type);
            println!("   Model name: {}", info.model_name);
            println!("   Typical latency: {}ms", info.typical_latency_ms);
            
            // Test direct command generation
            let request = cmdai::models::CommandRequest::new("list all pdf files size less than 5mb", ShellType::Bash);
            println!("   Testing command: '{}'", request.input);
            
            match embedded_backend.generate_command(&request).await {
                Ok(result) => {
                    println!("   ✅ Generated: '{}'", result.command);
                    println!("   Backend used: {}", result.backend_used);
                    println!("   Confidence: {:.2}", result.confidence_score);
                    println!("   Generation time: {}ms", result.generation_time_ms);
                }
                Err(e) => {
                    println!("   ❌ Error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create embedded backend: {:?}", e);
        }
    }
    
    // Test 2: Direct decision tree backend test
    println!("\n2. Testing Decision Tree Backend Directly:");
    println!("------------------------------------------");
    
    match DecisionTreeBackend::new() {
        Ok(decision_tree) => {
            println!("✅ Decision tree backend created successfully");
            
            let is_available = decision_tree.is_available().await;
            println!("   Available: {}", is_available);
            
            let info = decision_tree.backend_info();
            println!("   Backend type: {:?}", info.backend_type);
            println!("   Model name: {}", info.model_name);
            
            // Test command generation
            let request = cmdai::models::CommandRequest::new("list all pdf files size less than 5mb", ShellType::Bash);
            println!("   Testing command: '{}'", request.input);
            
            match decision_tree.generate_command(&request).await {
                Ok(result) => {
                    println!("   ✅ Generated: '{}'", result.command);
                    println!("   Backend used: {}", result.backend_used);
                    println!("   Confidence: {:.2}", result.confidence_score);
                }
                Err(e) => {
                    println!("   ❌ Error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create decision tree backend: {:?}", e);
        }
    }
    
    // Test 3: CLI App backend selection
    println!("\n3. Testing CLI App Backend Selection:");
    println!("-------------------------------------");
    
    let config = CliConfig {
        default_shell: ShellType::Bash,
        safety_level: SafetyLevel::Moderate,
        output_format: OutputFormat::Plain,
        auto_confirm: false,
    };
    
    match CliApp::with_config(config).await {
        Ok(cli_app) => {
            println!("✅ CLI app created successfully");
            
            // Test various commands to see which backend is selected
            let test_commands = vec![
                "list all pdf files size less than 5mb",
                "list all files",
                "show current directory",
                "find img files greater than 10mb",
            ];
            
            for command in test_commands {
                println!("\n   Testing: '{}'", command);
                
                let args = MockArgs {
                    prompt: Some(command.to_string()),
                    shell: None,
                    safety: None,
                    output: None,
                    confirm: false,
                    verbose: true, // Enable verbose for more info
                    config_file: None,
                };
                
                match cli_app.run_with_args(args).await {
                    Ok(result) => {
                        println!("   → Generated: '{}'", result.generated_command);
                        println!("   → Backend details: {}", result.generation_details);
                        println!("   → Generation time: {}ms", result.timing_info.generation_time_ms);
                        
                        // Check if this looks like decision tree fallback
                        if result.generated_command == "ls -la" || result.generated_command.starts_with("echo 'No pattern found") {
                            println!("   ⚠️  Looks like decision tree fallback was used");
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Error: {:?}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create CLI app: {:?}", e);
        }
    }
}

#[tokio::test]
async fn debug_embedded_backend_inference() {
    println!("\n=== Embedded Backend Inference Debug ===");
    
    // Test the inference pipeline step by step
    match EmbeddedModelBackend::new() {
        Ok(backend) => {
            println!("✅ Embedded backend created");
            
            // Check model path
            println!("   Model path: {:?}", backend.model_path());
            println!("   Variant: {:?}", backend.variant());
            
            // Test model loading
            println!("\n   Testing model loading...");
            // Note: We can't directly test load_model as it requires &mut, but we can test inference
            
            let request = cmdai::models::CommandRequest::new("list files", ShellType::Bash);
            println!("   Testing simple command: '{}'", request.input);
            
            let start = std::time::Instant::now();
            match backend.generate_command(&request).await {
                Ok(result) => {
                    let duration = start.elapsed();
                    println!("   ✅ Success in {:?}", duration);
                    println!("   Generated: '{}'", result.command);
                    println!("   Explanation: {}", result.explanation);
                    println!("   Backend: {}", result.backend_used);
                    
                    // If this doesn't look like a real LLM response, there's an issue
                    if result.command == "find . -name '*.txt'" || result.command == "ls -la" {
                        println!("   ⚠️  This looks like a fallback/mock response, not real LLM inference");
                    }
                }
                Err(e) => {
                    println!("   ❌ Failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create embedded backend: {:?}", e);
        }
    }
}

// Mock args helper
#[derive(Debug, Clone)]
struct MockArgs {
    pub prompt: Option<String>,
    pub shell: Option<String>,
    pub safety: Option<String>,
    pub output: Option<String>,
    pub confirm: bool,
    pub verbose: bool,
    pub config_file: Option<String>,
}

impl cmdai::cli::IntoCliArgs for MockArgs {
    fn prompt(&self) -> Option<String> { self.prompt.clone() }
    fn shell(&self) -> Option<String> { self.shell.clone() }
    fn safety(&self) -> Option<String> { self.safety.clone() }
    fn output(&self) -> Option<String> { self.output.clone() }
    fn confirm(&self) -> bool { self.confirm }
    fn verbose(&self) -> bool { self.verbose }
    fn config_file(&self) -> Option<String> { self.config_file.clone() }
}