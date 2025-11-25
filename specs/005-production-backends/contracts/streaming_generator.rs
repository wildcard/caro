// Contract Test: Streaming Command Generation System
// Tests real-time generation with cancellation and progress feedback

use cmdai::streaming::{StreamingGenerator, GenerationRequest, StreamEvent, GenerationResult};
use cmdai::backends::BackendType;
use anyhow::Result;
use tokio::time::{timeout, Duration};
use tokio_stream::StreamExt;

#[tokio::test]
async fn test_basic_streaming_generation() -> Result<()> {
    let generator = StreamingGenerator::new(BackendType::Mock).await?;
    
    let request = GenerationRequest {
        user_input: "list all files".to_string(),
        working_directory: "/home/user".to_string(),
        shell_type: "bash".to_string(),
        context: None,
    };
    
    let mut stream = generator.generate_stream(request).await?;
    let mut events = Vec::new();
    
    while let Some(event) = stream.next().await {
        events.push(event?);
        if matches!(events.last(), Some(StreamEvent::Completed { .. })) {
            break;
        }
    }
    
    // Verify we received progress updates and final result
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Progress { .. })));
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Completed { .. })));
    
    Ok(())
}

#[tokio::test]
async fn test_streaming_cancellation() -> Result<()> {
    let generator = StreamingGenerator::new(BackendType::Mock).await?;
    
    let request = GenerationRequest {
        user_input: "complex analysis task".to_string(),
        working_directory: "/home/user".to_string(),
        shell_type: "bash".to_string(),
        context: None,
    };
    
    let mut stream = generator.generate_stream(request).await?;
    let cancellation_token = stream.cancellation_token().clone();
    
    // Start generation
    let generation_task = tokio::spawn(async move {
        let mut events = Vec::new();
        while let Some(event) = stream.next().await {
            events.push(event?);
            if events.len() > 2 {
                break; // Simulate cancellation after some progress
            }
        }
        Ok::<_, anyhow::Error>(events)
    });
    
    // Cancel after short delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    cancellation_token.cancel();
    
    let events = generation_task.await??;
    
    // Should receive cancellation event
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Cancelled)));
    
    Ok(())
}

#[tokio::test]
async fn test_partial_result_handling() -> Result<()> {
    let generator = StreamingGenerator::new(BackendType::Mock).await?;
    
    let request = GenerationRequest {
        user_input: "find and process files".to_string(),
        working_directory: "/home/user".to_string(),
        shell_type: "bash".to_string(),
        context: None,
    };
    
    let mut stream = generator.generate_stream(request).await?;
    let mut partial_commands = Vec::new();
    
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::PartialResult { command, confidence } => {
                partial_commands.push((command, confidence));
            }
            StreamEvent::Completed { .. } => break,
            _ => {}
        }
    }
    
    // Verify partial results show progressive refinement
    assert!(!partial_commands.is_empty());
    
    // Confidence should generally increase over time
    if partial_commands.len() > 1 {
        let first_confidence = partial_commands[0].1;
        let last_confidence = partial_commands.last().unwrap().1;
        assert!(last_confidence >= first_confidence);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_generation_timeout() -> Result<()> {
    let generator = StreamingGenerator::new(BackendType::Mock).await?;
    
    let request = GenerationRequest {
        user_input: "very slow operation".to_string(),
        working_directory: "/home/user".to_string(),
        shell_type: "bash".to_string(),
        context: None,
    };
    
    // Set very short timeout to force timeout
    let result = timeout(
        Duration::from_millis(10),
        generator.generate_blocking(request)
    ).await;
    
    assert!(result.is_err()); // Should timeout
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_concurrent_streams() -> Result<()> {
    let generator = StreamingGenerator::new(BackendType::Mock).await?;
    
    // Start multiple concurrent generations
    let tasks = (0..3).map(|i| {
        let generator = generator.clone();
        tokio::spawn(async move {
            let request = GenerationRequest {
                user_input: format!("task number {}", i),
                working_directory: "/home/user".to_string(),
                shell_type: "bash".to_string(),
                context: None,
            };
            
            generator.generate_blocking(request).await
        })
    }).collect::<Vec<_>>();
    
    // Wait for all to complete
    let results = futures::future::try_join_all(tasks).await?;
    
    // All should complete successfully
    for result in results {
        assert!(result.is_ok());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_error_recovery() -> Result<()> {
    let generator = StreamingGenerator::new(BackendType::Mock).await?;
    
    let request = GenerationRequest {
        user_input: "trigger error".to_string(), // Special input to simulate error
        working_directory: "/home/user".to_string(),
        shell_type: "bash".to_string(),
        context: None,
    };
    
    let mut stream = generator.generate_stream(request).await?;
    let mut error_events = Vec::new();
    
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::Error { error, recoverable } => {
                error_events.push((error, recoverable));
                if !recoverable {
                    break;
                }
            }
            StreamEvent::Completed { .. } => break,
            _ => {}
        }
    }
    
    assert!(!error_events.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_progress_reporting_accuracy() -> Result<()> {
    let generator = StreamingGenerator::new(BackendType::Mock).await?;
    
    let request = GenerationRequest {
        user_input: "progress tracking test".to_string(),
        working_directory: "/home/user".to_string(),
        shell_type: "bash".to_string(),
        context: None,
    };
    
    let mut stream = generator.generate_stream(request).await?;
    let mut progress_values = Vec::new();
    
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::Progress { percentage, .. } => {
                progress_values.push(percentage);
            }
            StreamEvent::Completed { .. } => break,
            _ => {}
        }
    }
    
    // Progress should be monotonically increasing
    for window in progress_values.windows(2) {
        assert!(window[1] >= window[0], "Progress decreased from {} to {}", window[0], window[1]);
    }
    
    // Final progress should be 100%
    if let Some(&last_progress) = progress_values.last() {
        assert!((last_progress - 100.0).abs() < f64::EPSILON);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_streaming_performance_metrics() -> Result<()> {
    let generator = StreamingGenerator::new(BackendType::Mock).await?;
    
    let request = GenerationRequest {
        user_input: "performance test command".to_string(),
        working_directory: "/home/user".to_string(),
        shell_type: "bash".to_string(),
        context: None,
    };
    
    let start = std::time::Instant::now();
    let result = generator.generate_blocking(request).await?;
    let total_duration = start.elapsed();
    
    // Verify performance metrics are captured
    assert!(result.generation_time.as_millis() > 0);
    assert!(result.tokens_generated > 0);
    assert!(total_duration >= result.generation_time);
    
    // First response should be within 500ms (requirement)
    assert!(total_duration.as_millis() < 500, 
        "Generation took {}ms, expected <500ms", total_duration.as_millis());
    
    Ok(())
}