//! Contract Test: Streaming Generator
//! 
//! This test validates the real-time command generation system with
//! progress feedback, cancellation support, and partial results.
//! 
//! MUST FAIL: These tests expect implementation of production streaming system
//! that provides real-time generation with proper async streaming.

use cmdai::streaming::{StreamingGenerator, GenerationStream, StreamEvent, GenerationResult, ProgressTracker, CancellationToken};
use cmdai::backends::MockBackend;
use anyhow::Result;
use futures_util::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_streaming_generator_initialization() -> Result<()> {
    let backend = Box::new(MockBackend::new());
    let generator = StreamingGenerator::new(backend).await?;
    
    // Verify initialization
    assert!(generator.is_ready());
    assert!(!generator.is_generating());
    assert_eq!(generator.get_active_streams_count(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_basic_streaming_generation() -> Result<()> {
    let backend = Box::new(MockBackend::new());
    let generator = StreamingGenerator::new(backend).await?;
    
    let user_input = "list all files in current directory";
    let working_dir = "/home/user";
    
    // Start streaming generation
    let stream = generator.generate_stream(user_input, working_dir).await?;
    
    // Verify stream properties
    assert!(!stream.request_id.is_empty());
    assert!(!stream.is_completed());
    assert!(!stream.is_cancelled());
    
    // Collect stream events
    let mut events = Vec::new();
    let mut stream_pin = Box::pin(stream.stream);
    
    while let Some(event) = stream_pin.next().await {
        events.push(event);
        
        // Break on completion to avoid infinite streams in tests
        if matches!(events.last(), Some(StreamEvent::Completed { .. })) {
            break;
        }
    }
    
    // Verify we received expected events
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Progress { .. })));
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Completed { .. })));
    
    // Verify final result
    if let Some(StreamEvent::Completed { result }) = events.last() {
        assert!(!result.command.is_empty());
        assert!(!result.explanation.is_empty());
        assert!(result.confidence > 0.0);
        assert!(result.generation_time < Duration::from_secs(5));
    } else {
        panic!("Stream should end with Completed event");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_progress_tracking() -> Result<()> {
    let backend = Box::new(MockBackend::new());
    let generator = StreamingGenerator::new(backend).await?;
    
    let stream = generator.generate_stream("complex analysis task", "/home/user").await?;
    let mut stream_pin = Box::pin(stream.stream);
    
    let mut progress_events = Vec::new();
    
    while let Some(event) = stream_pin.next().await {
        if let StreamEvent::Progress { percentage, message } = &event {
            progress_events.push((percentage.clone(), message.clone()));
        }
        
        if matches!(event, StreamEvent::Completed { .. }) {
            break;
        }
    }
    
    // Verify progress tracking
    assert!(!progress_events.is_empty());
    
    // Progress should be monotonically increasing
    for window in progress_events.windows(2) {
        assert!(window[1].0 >= window[0].0, 
            "Progress should be monotonically increasing: {} -> {}", 
            window[0].0, window[1].0);
    }
    
    // Progress should start near 0 and end near 100
    assert!(progress_events.first().unwrap().0 <= 10.0);
    assert!(progress_events.last().unwrap().0 >= 90.0);
    
    // Messages should be descriptive
    for (_, message) in &progress_events {
        assert!(!message.is_empty());
        assert!(message.len() > 5); // Meaningful messages
    }
    
    Ok(())
}

#[tokio::test]
async fn test_partial_results() -> Result<()> {
    let backend = Box::new(MockBackend::new());
    let generator = StreamingGenerator::new(backend).await?;
    
    let stream = generator.generate_stream("find all python files and count lines", "/home/user").await?;
    let mut stream_pin = Box::pin(stream.stream);
    
    let mut partial_results = Vec::new();
    
    while let Some(event) = stream_pin.next().await {
        if let StreamEvent::PartialResult { command, confidence } = &event {
            partial_results.push((command.clone(), confidence.clone()));
        }
        
        if matches!(event, StreamEvent::Completed { .. }) {
            break;
        }
    }
    
    // Verify partial results show command refinement
    assert!(!partial_results.is_empty());
    
    // Confidence should generally increase over time
    let confidence_values: Vec<f64> = partial_results.iter().map(|(_, c)| *c).collect();
    let avg_first_half = confidence_values[..confidence_values.len()/2].iter().sum::<f64>() / (confidence_values.len()/2) as f64;
    let avg_second_half = confidence_values[confidence_values.len()/2..].iter().sum::<f64>() / (confidence_values.len() - confidence_values.len()/2) as f64;
    
    assert!(avg_second_half >= avg_first_half, "Confidence should improve over time");
    
    // Commands should become more specific/complete
    let first_command = &partial_results.first().unwrap().0;
    let last_command = &partial_results.last().unwrap().0;
    
    assert!(last_command.len() >= first_command.len(), 
        "Commands should become more detailed over time");
    
    Ok(())
}

#[tokio::test]
async fn test_stream_cancellation() -> Result<()> {
    let backend = Box::new(MockBackend::new());
    let generator = StreamingGenerator::new(backend).await?;
    
    let mut stream = generator.generate_stream("long running analysis", "/home/user").await?;
    let cancellation_token = stream.cancellation_token.clone();
    
    // Start consuming stream
    let stream_handle = tokio::spawn(async move {
        let mut events = Vec::new();
        let mut stream_pin = Box::pin(stream.stream);
        
        while let Some(event) = stream_pin.next().await {
            events.push(event.clone());
            
            if matches!(event, StreamEvent::Cancelled) {
                break;
            }
            
            // Prevent infinite loops in tests
            if events.len() > 100 {
                break;
            }
        }
        
        events
    });
    
    // Cancel after short delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    cancellation_token.cancel();
    
    // Wait for stream to complete
    let events = timeout(Duration::from_secs(2), stream_handle).await??;
    
    // Verify cancellation was processed
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Cancelled)), 
        "Stream should receive Cancelled event");
    
    // Should not receive Completed event after cancellation
    let cancelled_index = events.iter().position(|e| matches!(e, StreamEvent::Cancelled)).unwrap();
    for event in &events[cancelled_index..] {
        assert!(!matches!(event, StreamEvent::Completed { .. }), 
            "Should not receive Completed after Cancelled");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling_in_stream() -> Result<()> {
    let mut backend = MockBackend::new();
    backend.set_should_error(true); // Configure backend to simulate errors
    
    let generator = StreamingGenerator::new(Box::new(backend)).await?;
    let stream = generator.generate_stream("test command", "/home/user").await?;
    let mut stream_pin = Box::pin(stream.stream);
    
    let mut events = Vec::new();
    
    while let Some(event) = stream_pin.next().await {
        events.push(event.clone());
        
        if matches!(event, StreamEvent::Error { .. } | StreamEvent::Completed { .. }) {
            break;
        }
    }
    
    // Should receive error event
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Error { .. })), 
        "Should receive Error event when backend fails");
    
    // Check error details
    if let Some(StreamEvent::Error { error, recoverable }) = events.iter().find(|e| matches!(e, StreamEvent::Error { .. })) {
        assert!(!error.is_empty());
        assert!(recoverable == &true || recoverable == &false); // Should specify recoverability
    }
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_concurrent_streams() -> Result<()> {
    let backend = Box::new(MockBackend::new());
    let generator = std::sync::Arc::new(StreamingGenerator::new(backend).await?);
    
    let stream_count = 5;
    let mut handles = Vec::new();
    
    // Start multiple concurrent streams
    for i in 0..stream_count {
        let generator_clone = generator.clone();
        let handle = tokio::spawn(async move {
            let user_input = format!("test command {}", i);
            let stream = generator_clone.generate_stream(&user_input, "/home/user").await.unwrap();
            let stream_id = stream.request_id.clone();
            
            let mut stream_pin = Box::pin(stream.stream);
            let mut events = Vec::new();
            
            while let Some(event) = stream_pin.next().await {
                events.push(event);
                
                if matches!(events.last(), Some(StreamEvent::Completed { .. })) {
                    break;
                }
                
                // Prevent runaway streams in tests
                if events.len() > 50 {
                    break;
                }
            }
            
            (stream_id, events)
        });
        handles.push(handle);
    }
    
    // Wait for all streams to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = timeout(Duration::from_secs(10), handle).await??;
        results.push(result);
    }
    
    // Verify all streams completed successfully
    assert_eq!(results.len(), stream_count);
    
    for (stream_id, events) in results {
        assert!(!stream_id.is_empty());
        assert!(!events.is_empty());
        assert!(events.iter().any(|e| matches!(e, StreamEvent::Completed { .. })), 
            "Stream {} should complete", stream_id);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_stream_performance_requirements() -> Result<()> {
    let backend = Box::new(MockBackend::new());
    let generator = StreamingGenerator::new(backend).await?;
    
    // Test first response time (constitutional: <500ms)
    let start = std::time::Instant::now();
    let stream = generator.generate_stream("simple test", "/home/user").await?;
    let mut stream_pin = Box::pin(stream.stream);
    
    // Wait for first event
    let first_event = stream_pin.next().await;
    let first_response_time = start.elapsed();
    
    assert!(first_response_time.as_millis() < 500, 
        "First response time {} ms exceeds constitutional requirement of 500ms", 
        first_response_time.as_millis());
    
    assert!(first_event.is_some(), "Should receive first event within time limit");
    
    Ok(())
}

#[tokio::test]
async fn test_stream_resource_cleanup() -> Result<()> {
    let backend = Box::new(MockBackend::new());
    let generator = StreamingGenerator::new(backend).await?;
    
    // Create and complete a stream
    {
        let stream = generator.generate_stream("test cleanup", "/home/user").await?;
        let mut stream_pin = Box::pin(stream.stream);
        
        // Consume entire stream
        while let Some(_event) = stream_pin.next().await {
            // Continue until stream ends
        }
    } // Stream goes out of scope here
    
    // Verify resources are cleaned up
    tokio::time::sleep(Duration::from_millis(100)).await; // Allow cleanup time
    assert_eq!(generator.get_active_streams_count(), 0, 
        "All streams should be cleaned up after completion");
    
    Ok(())
}

#[tokio::test]
async fn test_stream_metadata_and_timing() -> Result<()> {
    let backend = Box::new(MockBackend::new());
    let generator = StreamingGenerator::new(backend).await?;
    
    let stream = generator.generate_stream("metadata test", "/home/user").await?;
    let mut stream_pin = Box::pin(stream.stream);
    
    let mut events = Vec::new();
    let stream_start = std::time::Instant::now();
    
    while let Some(event) = stream_pin.next().await {
        events.push((event, stream_start.elapsed()));
        
        if matches!(events.last().unwrap().0, StreamEvent::Completed { .. }) {
            break;
        }
    }
    
    // Verify timing metadata in final result
    if let Some((StreamEvent::Completed { result }, _)) = events.last() {
        assert!(result.tokens_generated > 0, "Should track token generation");
        assert!(result.generation_time > Duration::from_millis(0), "Should track generation time");
        assert!(result.safety_validated, "Should indicate safety validation occurred");
    }
    
    // Verify event timing progression
    for window in events.windows(2) {
        assert!(window[1].1 >= window[0].1, "Events should have monotonically increasing timestamps");
    }
    
    Ok(())
}