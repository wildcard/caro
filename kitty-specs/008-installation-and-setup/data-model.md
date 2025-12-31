# Data Model (Discovery Draft)

Use this skeleton to capture entities, attributes, and relationships uncovered during research. Update it as the solution space becomes clearer; implementation will refine and extend it.

## Entities

### Entity: InstallationMethod
- **Description**: Represents a way to install Caro (automated script, cargo, binary download, build from source, package manager)
- **Attributes**:
  - `name` (string) – Display name of the installation method (e.g., "Automated Script", "Cargo Install", "Homebrew")
  - `description` (string) – Brief explanation of what this method does
  - `difficulty` (enum: easy | intermediate | advanced) – Skill level required
  - `platforms` (array[Platform]) – Which platforms this method supports
  - `status` (enum: available | coming_soon) – Whether method is currently functional
  - `install_command` (string) – Primary command to run (e.g., "cargo install caro")
  - `verification_command` (string) – Command to verify successful installation (e.g., "caro --version")
  - `uninstall_command` (string) – Command to remove Caro
  - `prerequisites` (array[string]) – Required dependencies (e.g., "Rust 1.75+", "Git")
- **Identifiers**: `name` (unique)
- **Lifecycle Notes**: Created when documenting new installation method; updated when method becomes available; archived if deprecated

### Entity: Platform
- **Description**: Represents an operating system or distribution (macOS, Linux distros, Windows)
- **Attributes**:
  - `name` (string) – Platform name (e.g., "macOS", "Ubuntu", "Arch Linux")
  - `architecture` (array[string]) – Supported CPU architectures (e.g., ["x86_64", "arm64"])
  - `version_requirements` (string) – Minimum OS version if applicable (e.g., "macOS 12+")
  - `status` (enum: supported | coming_soon | unsupported) – Current support level
  - `notes` (string) – Platform-specific considerations
- **Identifiers**: `name` (unique)
- **Lifecycle Notes**: Created when platform is targeted; status updated as support is added/removed

### Entity: SetupOption
- **Description**: Represents a post-install configuration option (shell completions, aliases, backend config, tool integration)
- **Attributes**:
  - `name` (string) – Display name (e.g., "Shell Completions", "Backend Configuration")
  - `category` (enum: shell_integration | backend_config | tool_integration | workflow) – Type of setup
  - `description` (string) – What this setup option provides
  - `supported_shells` (array[Shell]) – Which shells this applies to (if shell-specific)
  - `status` (enum: available | coming_soon) – Whether option is currently functional
  - `setup_steps` (array[string]) – Ordered list of configuration steps
  - `config_file_path` (string) – Path to config file if applicable (e.g., "~/.config/caro/config.toml")
  - `example_config` (string) – Example configuration snippet
- **Identifiers**: `name` (unique)
- **Lifecycle Notes**: Created when documenting new setup option; updated when implementation changes

### Entity: Shell
- **Description**: Represents a command shell (bash, zsh, fish)
- **Attributes**:
  - `name` (string) – Shell name (e.g., "bash", "zsh", "fish")
  - `completion_syntax` (string) – How completions are generated/installed for this shell
  - `alias_syntax` (string) – How to define aliases in this shell
  - `config_file` (string) – Default configuration file (e.g., "~/.bashrc", "~/.zshrc")
  - `priority` (integer) – Display order on documentation page (1=highest priority)
- **Identifiers**: `name` (unique)
- **Lifecycle Notes**: Static entity; rarely changes unless new shell support is added

### Entity: DocumentationPage
- **Description**: Represents one of the three documentation pages being created
- **Attributes**:
  - `page_id` (enum: quick_start | installation | setup) – Unique page identifier
  - `title` (string) – Page title (e.g., "Quick Start Guide")
  - `url_path` (string) – URL path (e.g., "/quick-start", "/installation")
  - `target_audience` (string) – Primary audience description
  - `content_sections` (array[string]) – Ordered list of section names
  - `nav_icon` (string) – Emoji or icon for navigation menu
  - `nav_description` (string) – Short description for dropdown menu
- **Identifiers**: `page_id` (unique)
- **Lifecycle Notes**: Created during implementation; content evolves but structure remains stable

## Relationships

| Source | Relation | Target | Cardinality | Notes |
|--------|----------|--------|-------------|-------|
| InstallationMethod | supports | Platform | M:N | One method can support multiple platforms; one platform can have multiple methods |
| SetupOption | applies_to | Shell | M:N | Some setup options are shell-specific (completions), others are universal (backend config) |
| DocumentationPage | contains | InstallationMethod | 1:N | Installation page lists all installation methods |
| DocumentationPage | contains | SetupOption | 1:N | Setup page lists all setup options |
| Platform | requires | InstallationMethod | N:M | Each platform has at least one installation method; methods may have platform-specific variations |

## Validation & Governance

- **Data quality requirements**:
  - All `status` fields must be either "available" or "coming_soon" (no other values)
  - `install_command` must be non-empty for methods with status "available"
  - `verification_command` should exist for all available installation methods
  - Platform `architecture` must include at least one valid architecture

- **Compliance considerations**:
  - No PII or sensitive data in this domain model
  - Installation commands should never include hardcoded secrets or credentials
  - External URLs (for binary downloads) should use HTTPS

- **Source of truth**:
  - InstallationMethod data sourced from README.md, project documentation, and CI/CD scripts
  - Platform data sourced from Cargo.toml platform targets and CI matrix
  - SetupOption data sourced from Caro CLI help text, config file schemas
  - Shell data sourced from shell documentation and completion generation code

> Treat this as a working model. When research uncovers new flows or systems, update the entities and relationships immediately so the implementation team inherits up-to-date context.
