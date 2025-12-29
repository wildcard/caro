---
applyTo: "**/*.md,docs/**/*"
---

# Documentation Review Instructions

Apply these guidelines when reviewing documentation and markdown files.

## Accuracy Requirements

### Claims Must Match Implementation
```markdown
<!-- BAD: Claim doesn't match reality -->
The setup script handles all prerequisites including Rust installation.
<!-- But actual method is `cargo install caro` which requires Rust -->

<!-- GOOD: Accurate description -->
Install with `cargo install caro` (requires Rust toolchain).
To install Rust, visit https://rustup.rs
```

### Numeric Claims Must Be Verified
```markdown
<!-- BAD: Unverified count -->
Includes 52+ dangerous command patterns

<!-- GOOD: Verified against codebase -->
Includes 49 predefined safety patterns  <!-- Matches patterns.rs -->
```

### Command Examples Must Be Correct
```markdown
<!-- BAD: Example doesn't match description -->
To show the top 5 processes by CPU:
```bash
ps aux --sort=-%cpu | head -6  # Shows 6 lines, not 5!
```

<!-- GOOD: Accurate example -->
To show the top 5 processes by CPU:
```bash
ps aux --sort=-%cpu | head -6  # 5 processes + header row
```
<!-- Or explain the discrepancy -->
```

## Path and URL Consistency

### Use Consistent Formats Within Files
```markdown
<!-- BAD: Mixed URL formats in same file -->
See [Contributing](/contribute)
See [License](/blob/main/LICENSE)

<!-- GOOD: Consistent format -->
See [Contributing](https://github.com/org/repo/contribute)
See [License](https://github.com/org/repo/blob/main/LICENSE)

<!-- OR use consistent relative paths -->
See [Contributing](./CONTRIBUTING.md)
See [License](./LICENSE)
```

### Relative vs Absolute Paths
```markdown
<!-- BAD: Inconsistent path references -->
See `../sessions/CELEBRATION.md`
See `/docs/guides/setup.md`

<!-- GOOD: Pick one style and stick with it -->
<!-- For cross-directory references, prefer root-relative -->
See `/docs/sessions/CELEBRATION.md`
See `/docs/guides/setup.md`
```

## Temporal Language

### Avoid "Just" and "Recently" Without Dates
```markdown
<!-- BAD: Becomes stale quickly -->
Spring Framework just adopted DCO in 2025.

<!-- GOOD: Factual statement -->
Spring Framework adopted DCO in January 2025.

<!-- ALSO GOOD: Relative to a known event -->
Spring Framework adopted DCO, joining other major projects
like the Linux kernel in using this approach.
```

### Date Format Consistency
```markdown
<!-- BAD: Inconsistent date formats -->
Added on 12/21/2025
Updated: December 21st, 2025
Last modified: 2025-12-21

<!-- GOOD: Pick one format -->
Added: December 21, 2025
Updated: December 21, 2025
Last modified: December 21, 2025

<!-- For technical docs, ISO format is often preferred -->
Added: 2025-12-21
```

## Terminology Precision

### Use Accurate Technical Terms
```markdown
<!-- BAD: Imprecise terminology -->
52 pre-compiled safety patterns
<!-- These are regex patterns, not compiled binaries -->

<!-- GOOD: Precise terminology -->
52 predefined safety patterns
<!-- Or if they are actually compiled at runtime: -->
52 regex patterns compiled at startup
```

### Consistent Naming
```markdown
<!-- BAD: Inconsistent product naming -->
Use caro to generate commands.
Caro provides safety validation.
The CARO tool is fast.

<!-- GOOD: Consistent casing -->
Use caro to generate commands.
caro provides safety validation.
The caro tool is fast.
```

## FAQ and Section Headings

### Headings Must Cover Content
```markdown
<!-- BAD: Heading doesn't match content -->
## Questions About the CLA?
<!-- But section discusses both CLA and DCO -->

<!-- GOOD: Accurate heading -->
## Questions About Contributor Agreements?
```

### FAQ Coverage
```markdown
<!-- BAD: FAQ only covers one option when multiple exist -->
## FAQ
Q: What if I can't sign the CLA?
A: Contact maintainers.
<!-- Missing: DCO questions when both methods are offered -->

<!-- GOOD: Comprehensive FAQ -->
## FAQ
Q: What if I can't sign the CLA?
A: You can use DCO instead with `git commit -s`.

Q: What if I forget to sign off?
A: You can amend: `git commit --amend -s`

Q: Can I switch between CLA and DCO?
A: Yes, you can use either method for different contributions.
```

## Code Examples

### Ensure Examples Are Runnable
```markdown
<!-- BAD: Non-working example -->
```rust
let result = async_fn().await;  // Won't compile in sync context
```

<!-- GOOD: Complete, runnable example -->
```rust
#[tokio::main]
async fn main() {
    let result = async_fn().await;
    println!("{:?}", result);
}
```
```

### Include Expected Output When Helpful
```markdown
<!-- GOOD: Shows what to expect -->
```bash
$ caro "list files in current directory"
Generated command:
  ls -la

Execute? (y/N)
```
```

## Link Validation

### Check Links Are Valid
- Relative links must point to existing files
- External links should use HTTPS
- Anchor links must match actual headings

```markdown
<!-- BAD: Broken anchor link -->
See [Installation](#install)  <!-- But heading is "## Installing" -->

<!-- GOOD: Matches heading exactly -->
See [Installing](#installing)
```

## Legacy References

### Flag Outdated References
```markdown
<!-- FLAG: Old project name -->
Install caro with...

<!-- Should be updated to -->
Install caro with...
```

> **Important**: The project was renamed from `caro` to `caro` in December 2025. Flag any remaining references to the old name.

## README Structure

### Recommended Sections
1. Project name and description
2. Installation
3. Quick Start / Usage
4. Features
5. Configuration
6. Contributing
7. License

### Badges Should Be Current
```markdown
<!-- BAD: Outdated or broken badges -->
![Build](https://old-ci.example.com/badge)

<!-- GOOD: Current and working -->
![Build](https://github.com/org/repo/actions/workflows/ci.yml/badge.svg)
```

## Changelog Format

### Use Consistent Changelog Format
```markdown
## [Unreleased]
### Added
- New feature X

### Changed
- Updated behavior of Y

### Fixed
- Bug in Z

### Removed
- Deprecated feature W
```

## Accessibility

### Alt Text for Images
```markdown
<!-- BAD: Missing alt text -->
![](screenshot.png)

<!-- GOOD: Descriptive alt text -->
![Terminal showing caro generating a safe command](screenshot.png)
```

### Use Descriptive Link Text
```markdown
<!-- BAD: Non-descriptive -->
Click [here](docs/setup.md) for setup instructions.

<!-- GOOD: Descriptive -->
See the [setup instructions](docs/setup.md) to get started.
```
