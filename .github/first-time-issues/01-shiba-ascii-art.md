# 🐕 Add Shiba-themed ASCII Art to Welcome Message

**Labels**: `good-first-issue`, `first-time-contributor`, `ui`, `fun`, `caro-the-shiba`
**Difficulty**: Easy ⭐
**Skills**: Basic Rust, ASCII art, terminal output
**Perfect for**: Dog lovers, cat lovers, UI enthusiasts, anyone who thinks terminals should be more fun!

## The Vision

Our mascot is **Caro, a Shiba Inu** (they're basically cat-dogs!). Right now, when users run `cmdai --version` or first install the tool, they just see boring text. Let's change that!

We want to greet users with beautiful ASCII art of a Shiba Inu, making cmdai feel friendly and welcoming from the very first interaction.

## What You'll Build

Add ASCII art of a Shiba/dog that displays:
- On first run of `cmdai` (welcome message)
- When running `cmdai --about` (new flag)
- Optionally: Easter egg when running `cmdai "good dog"`

## Example ASCII Art

```
         __
    (___()'`;   Woof! I'm Caro, your terminal safety companion!
    /,    /`      Let's generate some commands safely together.
    \\"--\\
```

Or create your own! Shiba features:
- Pointy ears
- Curled tail
- Friendly expression
- Compact size (fits in terminal)

## Implementation Guide

### Step 1: Create the ASCII Art Module

Create a new file `src/ascii_art.rs`:

```rust
/// ASCII art for Caro the Shiba mascot
pub const CARO_ASCII: &str = r#"
         __
    (___()'`;
    /,    /`
    \\"--\\
"#;

pub fn display_welcome() {
    println!("{}", CARO_ASCII);
    println!("  Woof! I'm Caro, your terminal safety companion!");
}
```

### Step 2: Add the Module to Main

In `src/lib.rs`, add:
```rust
pub mod ascii_art;
```

### Step 3: Call It in the CLI

In `src/main.rs`, update the CLI to show the welcome message:
- On `--about` flag
- On first run (check for config file existence)

### Step 4: Add Tests

Create tests in `src/ascii_art.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caro_ascii_is_not_empty() {
        assert!(!CARO_ASCII.is_empty());
    }

    #[test]
    fn test_caro_ascii_is_multiline() {
        assert!(CARO_ASCII.contains('\n'));
    }
}
```

## Acceptance Criteria

- [ ] ASCII art of a Shiba/dog is added to the codebase
- [ ] Art displays on `cmdai --about` command
- [ ] Art is displayed on first run (when no config exists)
- [ ] Welcome message includes friendly greeting from Caro
- [ ] ASCII art fits within 80 columns (terminal-friendly)
- [ ] Tests verify the ASCII art exists and is valid
- [ ] Code is formatted with `cargo fmt`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`

## Resources

- [ASCII Art Generator](https://www.asciiart.eu/animals/dogs)
- [Shiba Inu Photos](https://unsplash.com/s/photos/shiba-inu) for inspiration
- Example: Look at how `cargo` displays its logo with ASCII art

## Why This Matters

First impressions matter! A friendly ASCII mascot:
- Makes the tool feel approachable
- Reinforces our brand identity (Caro the Shiba)
- Adds personality to CLI interactions
- Shows users we care about UX, even in terminals

## Questions?

Ask in the comments! We'll help you with:
- Where to add the code
- How to test terminal output
- ASCII art design feedback
- Anything else you need!

## Pro Tips for First-Timers

1. **Test your ASCII art** in your actual terminal before committing
2. **Keep it simple** - complex art might not render well on all terminals
3. **Consider color** - We can add colors later with the `colored` crate
4. **Have fun!** This is a feature that brings joy to users

**Welcome to the team! Let's make cmdai more adorable together! 🐕**
