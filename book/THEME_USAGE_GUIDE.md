# cmdai Documentation Theme - Usage Guide

This guide explains how to use the new theme features in your documentation.

## Using Callout Boxes

To add styled callout boxes to your documentation, use HTML blocks with the appropriate CSS classes:

### Info Box (Blue)
```html
<div class="info">
<strong>Info</strong>
This is informational content that users should know about.
</div>
```

### Warning Box (Orange)
```html
<div class="warning">
<strong>Warning</strong>
This highlights important warnings or cautions.
</div>
```

### Danger Box (Red)
```html
<div class="danger">
<strong>Danger</strong>
This indicates critical or dangerous operations.
</div>
```

### Success Box (Green)
```html
<div class="success">
<strong>Success</strong>
This indicates successful operations or positive outcomes.
</div>
```

### Tip Box (Cyan)
```html
<div class="tip">
<strong>Tip</strong>
Helpful tips and tricks for users.
</div>
```

### Note Box (Gray)
```html
<div class="note">
<strong>Note</strong>
General notes and additional information.
</div>
```

## Using Status Badges

### Inline Badges
```html
<span class="badge badge-success">Success</span>
<span class="badge badge-warning">Warning</span>
<span class="badge badge-danger">Danger</span>
<span class="badge badge-info">Info</span>
<span class="badge badge-tip">Tip</span>
```

### Status Badges for Features
```html
<span class="status-badge status-completed">Completed</span>
<span class="status-badge status-inprogress">In Progress</span>
<span class="status-badge status-planned">Planned</span>
```

## Code Block Features

### With Copy Button
When you include code blocks in markdown, copy buttons are automatically added:

```bash
cmdai "list all PDF files"
```

The copy button appears on hover and shows "Copied!" feedback.

### Language Labels
Code blocks with language specifications show the language label:

```rust
fn main() {
    println!("Hello, world!");
}
```

## Tables

Tables are automatically made responsive on mobile devices:

| Feature | Status | Details |
|---------|--------|---------|
| Copy to clipboard | Active | Automatic buttons on code blocks |
| Smooth scrolling | Active | Smooth anchor link scrolling |
| Mobile nav toggle | Active | Menu toggle on mobile devices |

## Example: Combining Features

Here's a complete example combining multiple theme features:

```markdown
# My Feature

<div class="tip">
<strong>Getting Started</strong>
This feature is <span class="badge badge-success">Production Ready</span>
</div>

Here's how to use it:

```bash
cmdai "your description here"
```

<div class="note">
<strong>Note</strong>
Be sure to review the safety guidelines before using this feature.
</div>

| Command | Purpose |
|---------|---------|
| `cmdai --help` | Show help |
| `cmdai --version` | Show version |
```

## CSS Classes Available

### Callout Boxes
- `.info` - Information
- `.warning` - Warnings
- `.danger` - Critical/Dangerous
- `.success` - Success states
- `.tip` - Tips and tricks
- `.note` - General notes

### Badges
- `.badge` - Base badge class
- `.badge-info` - Info badge
- `.badge-success` - Success badge
- `.badge-warning` - Warning badge
- `.badge-danger` - Danger badge
- `.badge-tip` - Tip badge

### Status Badges
- `.status-badge` - Base status class
- `.status-completed` - Green completed state
- `.status-inprogress` - Orange in-progress state
- `.status-planned` - Blue planned state

### Tables
- `.table-wrapper` - Auto-applied wrapper for responsive tables

### Custom Elements
- `.breadcrumb` - Breadcrumb navigation
- `.copy-button` - Code copy buttons (auto-applied)
- `.code-language` - Language labels on code blocks

## JavaScript Features (Automatic)

These features work automatically without any configuration:

1. **Copy to Clipboard** - Buttons appear on all code blocks
2. **Smooth Scrolling** - Anchor links scroll smoothly
3. **Table Responsiveness** - Tables wrap automatically for mobile
4. **Mobile Menu Toggle** - Menu becomes toggleable on mobile
5. **Heading Anchors** - All headings with IDs become clickable
6. **Scroll Progress Bar** - Shows reading progress (desktop only)
7. **Dark Mode Detection** - Respects system theme preference

## Browser Support

All features work in:
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Mobile browsers (iOS Safari, Chrome Mobile)

Older browsers receive graceful degradation - all content remains readable.

## Mobile Responsive Breakpoints

- **Desktop**: Full layout with all features
- **Tablet (768px - 1024px)**: Sidebar toggle, optimized tables
- **Mobile (< 768px)**: Single column, responsive fonts, touch-friendly
- **Small Mobile (< 480px)**: Extra-small optimizations

## Accessibility

The theme is designed with accessibility in mind:
- Keyboard navigation fully supported
- High contrast mode support
- Reduced motion preference respected
- Proper heading hierarchy
- WCAG 2.1 AA compliant

## Tips for Best Results

1. **Callout Boxes**: Use sparingly, only when emphasis is needed
2. **Code Examples**: Always include language specification
3. **Tables**: Keep columns reasonable for mobile viewing
4. **Links**: Use descriptive link text (avoid "click here")
5. **Images**: Include alt text for accessibility
6. **Headings**: Use proper hierarchy (don't skip levels)

## Troubleshooting

### Copy button not appearing?
- Ensure code block is properly formatted
- Check browser console for JavaScript errors

### Tables look cramped on mobile?
- This is intentional - tables scroll horizontally on mobile
- Consider simplifying complex tables

### Dark mode not switching?
- Check system preferences
- Try manual theme selector

## Questions?

See the main documentation or check the GitHub repository for more information.
