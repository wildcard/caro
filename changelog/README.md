# Caro Changelog Website

A stunning changelog and release notes website for [Caro](https://caro.sh) - your loyal shell companion.

Built with [Astro](https://astro.build/) and inspired by the [Starlog](https://github.com/withastro/astro/tree/main/examples/starlog) template.

## Features

- Beautiful, responsive design with dark mode support
- Automatic content generation from `CHANGELOG.md`
- Individual release pages with deep linking
- SEO optimized with Open Graph tags
- Fast performance with Astro's static site generation
- Teal/coral color scheme matching Caro's brand

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Syncing from CHANGELOG.md

The changelog content can be automatically generated from the root `CHANGELOG.md` file:

```bash
npm run sync-changelog
```

This will parse the changelog and generate individual release markdown files in `src/content/releases/`.

## Structure

```
changelog/
├── public/              # Static assets
├── src/
│   ├── components/      # Astro components
│   ├── content/
│   │   └── releases/    # Release markdown files
│   ├── layouts/         # Page layouts
│   ├── pages/           # Route pages
│   └── styles/          # SCSS styles
├── scripts/
│   └── sync-changelog.mjs  # CHANGELOG.md parser
└── astro.config.mjs
```

## Deployment

The site is designed to be deployed at `https://changelog.caro.sh`. Configure your hosting provider to point to the `dist/` directory after running `npm run build`.

## License

AGPL-3.0 - See [LICENSE](../LICENSE) for details.
