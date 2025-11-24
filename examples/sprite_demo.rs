//! Demo of sprite animation rendering capabilities
//!
//! Run with: cargo run --example sprite_demo

use cmdai::rendering::{examples::*, Animation, AnimationMode, Animator};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== cmdai Sprite Animation Demo ===\n");

    let animator = Animator::new();

    // Demo 1: Static sprite (idle character)
    println!("1. Static Sprite - Idle Character");
    println!("Press Enter to continue...");
    wait_for_enter();

    let idle_sprite = create_idle_character()?;
    animator.render_static(&idle_sprite)?;
    println!();

    // Demo 2: Walking animation (loop 2 times)
    println!("\n2. Walking Animation (2 loops)");
    println!("Press Enter to start...");
    wait_for_enter();

    let walking_sprite = create_walking_animation()?;
    let mut walking_anim = Animation::new(walking_sprite, AnimationMode::LoopN(2));
    animator.play(&mut walking_anim).await?;

    // Demo 3: Heart pulse animation (loop forever until user stops)
    println!("\n3. Heart Pulse Animation (3 loops)");
    println!("Press Enter to start...");
    wait_for_enter();

    let heart_sprite = create_heart_animation()?;
    let mut heart_anim = Animation::new(heart_sprite, AnimationMode::LoopN(3));
    animator.play(&mut heart_anim).await?;

    // Demo 4: Spinning coin (loop 3 times)
    println!("\n4. Spinning Coin Animation (3 loops)");
    println!("Press Enter to start...");
    wait_for_enter();

    let coin_sprite = create_coin_animation()?;
    let mut coin_anim = Animation::new(coin_sprite, AnimationMode::LoopN(3));
    animator.play(&mut coin_anim).await?;

    // Demo 5: Loading spinner (infinite loop for 5 seconds)
    println!("\n5. Loading Spinner (5 seconds)");
    println!("Press Enter to start...");
    wait_for_enter();

    let spinner_sprite = create_spinner_animation()?;
    let mut spinner_anim = Animation::new(spinner_sprite, AnimationMode::LoopN(10));
    animator.play(&mut spinner_anim).await?;

    println!("\n\n=== Demo Complete! ===");
    println!("\nThe rendering system supports:");
    println!("  - Color palettes with hex colors");
    println!("  - Multiple animation frames with timing");
    println!("  - Transparent pixels");
    println!("  - Unicode block characters (█) for pixels");
    println!("  - True color (24-bit RGB) or 256-color ANSI");
    println!("  - Animation modes: Once, Loop, LoopN");

    Ok(())
}

fn wait_for_enter() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
}
