# cmdai Documentation

This directory contains the mdBook-based documentation for cmdai.

## Building the Documentation

### Prerequisites

Install mdBook:

```bash
cargo install mdbook
```

### Build

```bash
# Build the documentation
mdbook build

# Serve locally with live reload
mdbook serve

# Open in browser
mdbook serve --open
```

The built documentation will be in `book/` directory.

## Structure

```
book/
├── book.toml              # mdBook configuration
├── src/                   # Documentation source
│   ├── SUMMARY.md        # Table of contents
│   ├── introduction.md   # Landing page
│   ├── user-guide/       # User documentation
│   ├── dev-guide/        # Developer documentation
│   ├── technical/        # Technical deep dives
│   ├── reference/        # Reference documentation
│   └── community/        # Community resources
└── theme/                # Custom CSS and assets
```

## Deployment

Documentation is automatically deployed to GitHub Pages when changes are pushed to the `main` branch.

The deployment is handled by the GitHub Actions workflow in `.github/workflows/docs.yml`.

## Local Development

```bash
# Start development server with auto-reload
mdbook serve

# The site will be available at http://localhost:3000
```

## Adding New Pages

1. Create a new `.md` file in the appropriate `src/` subdirectory
2. Add an entry to `src/SUMMARY.md`
3. Build to verify: `mdbook build`
4. Commit and push

## Customization

- **Configuration**: Edit `book.toml`
- **Theme**: Modify `theme/custom.css`
- **Structure**: Update `src/SUMMARY.md`

## Links

- **Live Documentation**: https://wildcard.github.io/cmdai (after first deployment)
- **mdBook Documentation**: https://rust-lang.github.io/mdBook/
- **Main Repository**: https://github.com/wildcard/cmdai
