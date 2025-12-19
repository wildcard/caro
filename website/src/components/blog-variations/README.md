# Blog Component Variations

This directory contains multiple blog component variations that provide different design approaches for displaying blog posts.

## Available Variations

### 1. ClassicCard
**Design**: Traditional card-based layout with hover effects
- Grid layout with responsive columns
- Hover animations with lift and shadow effects
- Date and read time in header
- Excerpt truncation
- Best for: Standard blog listings

### 2. MagazineStyle
**Design**: Editorial layout with large featured images
- Full-width cards with image placeholders
- First post can be featured with larger sizing
- Magazine-style typography
- Best for: Visual-heavy blogs, featured stories

### 3. MinimalistList
**Design**: Clean, text-focused minimal design
- Single-column list layout
- Horizontal line separators
- Subtle hover animations (slide-in effect)
- Uppercase metadata styling
- Best for: Text-heavy content, simple aesthetic

### 4. Timeline
**Design**: Vertical timeline with date emphasis
- Visual timeline with connecting line
- Date displayed in dedicated box
- Animated entry on load
- Chronological emphasis
- Best for: Release notes, changelogs, historical content

### 5. GridMasonry
**Design**: Pinterest-style masonry layout
- Variable height cards for visual variety
- Icon-based image placeholders
- Rotating icon animations on hover
- Compact, efficient use of space
- Best for: Multiple posts, diverse content

## Usage

### In Blog.astro Component

Update the `blogPosts` array to include a `variation` field for each post:

```astro
const blogPosts = [
  {
    title: "Your Post Title",
    slug: "your-post-slug",
    date: "2025-12-17",
    excerpt: "Brief description of your post...",
    readTime: "5 min read",
    variation: "classic" // Options: classic, magazine, minimalist, timeline, masonry
  }
];
```

The variation is automatically selected based on the first post's `variation` field.

### In Individual Blog Posts

Each blog post can also have its own page variation. In the blog post file:

```astro
<BlogPost
  title="Your Post Title"
  description="Post description"
  date="2025-12-17"
  readTime="8 min read"
  variation="story" // Options: default, minimal, feature, technical, story
>
  <!-- Your content here -->
</BlogPost>
```

## Blog Post Page Variations

### default
Standard blog post layout with gradient header

### minimal
Simplified layout with solid header and centered content (max-width: 700px)

### feature
Bold design with full gradient header background
- Orange gradient header (#ff8c42 to #ff6b35)
- White text on header
- Larger title (52px)
- Best for: Important announcements, featured posts

### technical
Enhanced code styling for technical content
- Dark code backgrounds
- Highlighted code blocks
- Bordered pre elements
- Best for: Technical tutorials, code examples

### story
Magazine-style storytelling layout
- Drop cap on first paragraph
- Serif italic title
- Best for: Narrative content, personal stories

## Adding New Variations

To add a new variation:

1. Create a new `.astro` file in this directory
2. Follow the existing component pattern with a `posts` prop
3. Import and add to `variationComponents` map in `Blog.astro`
4. Update this README with the new variation details

## Example: Creating a New Variation

```astro
---
// MyNewVariation.astro
interface Props {
  posts: Array<{
    title: string;
    slug: string;
    date: string;
    excerpt: string;
    readTime: string;
  }>;
}

const { posts } = Astro.props;
---

<div class="my-variation">
  {posts.map(post => (
    <article class="my-card">
      <!-- Your custom layout here -->
    </article>
  ))}
</div>

<style>
  /* Your custom styles here */
</style>
```

Then update `Blog.astro`:

```astro
import MyNewVariation from './blog-variations/MyNewVariation.astro';

const variationComponents = {
  classic: ClassicCard,
  magazine: MagazineStyle,
  minimalist: MinimalistList,
  timeline: Timeline,
  masonry: GridMasonry,
  mynew: MyNewVariation  // Add your new variation
};
```

## Design Guidelines

When creating new variations, follow these guidelines:

1. **Responsive**: All variations must work on mobile, tablet, and desktop
2. **Accessible**: Use semantic HTML and proper ARIA labels
3. **Consistent**: Use the existing color palette and design tokens
4. **Performant**: Avoid heavy animations or large assets
5. **Theme-aware**: Use CSS variables for colors to support dark mode

## Color Palette

- Primary: `#ff8c42` (Caro Orange)
- Secondary: `#ff6b35`
- Background: `var(--color-bg)`
- Text: `var(--color-text)`
- Border: `var(--color-border)`
- Links: `var(--color-links-muted)`

## Testing Variations

To test a variation:

1. Update the `variation` field in `Blog.astro`
2. Run the development server: `npm run dev`
3. Navigate to the homepage and scroll to the blog section
4. Verify responsive behavior at different screen sizes
5. Test hover states and animations
6. Check dark mode compatibility

## Questions?

For questions or suggestions about blog variations, see the main project documentation or open an issue on GitHub.
