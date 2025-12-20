# Caro.sh Image Generation Prompts

## Reference Character Description

**Caro/Kyaro** is a Shiba Inu dog rendered in 8-bit pixel art style with:
- **Colors**: Black and cream/tan fur (Black & Tan Shiba coloring)
- **Style**: Retro 8-bit pixel art, clean pixels, no anti-aliasing
- **Pose**: Sitting, alert, friendly expression
- **Details**: White chest/bib marking, pointed ears, curled tail
- **Accessory**: Speech bubble (representing terminal/command output)
- **Mood**: Friendly, helpful, loyal companion aesthetic

---

## Required Images

### 1. Open Graph Image (Primary Social Share)

**Filename**: `og-image.png`
**Dimensions**: 1200 x 630 pixels
**Use**: Facebook, LinkedIn, Discord, Slack, iMessage link previews

**Prompt for Imagen 3**:
```
8-bit pixel art style social media banner featuring a cute black and tan Shiba Inu dog (Caro) sitting in the center. The dog has cream and black fur coloring with a white chest marking. Dark terminal/code editor background with subtle green terminal text patterns. The text "Caro" in large pixel font above the dog, and "Your Loyal Shell Companion" as a tagline below. Retro gaming aesthetic, clean pixel art, no gradients, limited color palette of dark grays, greens, orange-tan, black, and white. Professional tech product banner style. 1200x630 aspect ratio.
```

**Alternative Prompt (Simpler)**:
```
Pixel art banner, 8-bit retro style. Cute Shiba Inu dog with black and tan fur sitting centered. Dark background resembling a terminal screen with faint code. Text reads "Caro - Your Loyal Shell Companion" in pixel font. Colors: dark gray background, green accents, cream/orange dog, white highlights. Clean pixels, no anti-aliasing. Tech startup aesthetic meets retro gaming.
```

---

### 2. Twitter/X Card Image

**Filename**: `twitter-card.png`
**Dimensions**: 1200 x 628 pixels
**Use**: Twitter/X link previews (summary_large_image card)

**Prompt for Imagen 3**:
```
8-bit pixel art Twitter card image. Black and tan Shiba Inu dog in pixel art style, sitting with alert friendly expression. Terminal/command line themed dark background with subtle $ prompt symbols. Dog positioned left-center with space for implied text area on right. Speech bubble coming from dog showing "> _" cursor. Retro 8-bit aesthetic, clean pixels, dark theme with green and orange accent colors. Professional developer tool branding. 1200x628 pixels aspect ratio.
```

---

### 3. Apple Touch Icon

**Filename**: `apple-touch-icon.png`
**Dimensions**: 180 x 180 pixels
**Use**: iOS home screen icon, Safari bookmarks

**Prompt for Imagen 3**:
```
App icon, 8-bit pixel art style. Close-up of cute Shiba Inu dog face, black and tan coloring with cream markings. Friendly expression with pointed ears visible. Dark charcoal gray background. Extremely simple, recognizable at small sizes. Clean pixel art, limited colors: black, cream/tan, white, dark gray background. Square format, suitable for app icon. No text, just the dog face. 180x180 pixels.
```

**Alternative Prompt**:
```
Pixel art app icon. Shiba Inu dog head, front-facing, 8-bit retro game style. Black and tan fur colors with white chest peek. Minimal detail, bold shapes, recognizable at small size. Solid dark background. No gradients, clean pixels only. Square icon format.
```

---

### 4. Favicon 32x32

**Filename**: `favicon-32x32.png`
**Dimensions**: 32 x 32 pixels
**Use**: Browser tab icon, bookmarks bar

**Prompt for Imagen 3**:
```
Tiny pixel art favicon, 32x32 pixels. Extremely simplified Shiba Inu dog head or full sitting dog silhouette. Black, tan/orange, and white pixels only on transparent or dark background. Must be recognizable at very small size. 8-bit retro style, maximum 4-5 colors. Clean sharp pixels, no blur or anti-aliasing.
```

**Note**: At 32x32, consider just the dog's face or a very simplified sitting silhouette. May need manual pixel editing for best results.

---

### 5. Favicon 16x16

**Filename**: `favicon-16x16.png`
**Dimensions**: 16 x 16 pixels
**Use**: Smallest browser favicon

**Prompt for Imagen 3**:
```
Micro pixel art, 16x16 pixels only. Abstract Shiba Inu dog representation - just the essential shape. Orange/tan and black pixels suggesting a dog face or sitting dog. Transparent or solid dark background. Absolute minimum detail, must read as "dog" at tiny size. 3-4 colors maximum.
```

**Note**: This size is extremely small. Recommend manually creating or heavily editing AI output. Consider just an abstract "C" in pixel style or simplified dog silhouette.

---

## Color Palette Reference

Based on the existing Caro pixel art:

| Color | Hex | Use |
|-------|-----|-----|
| Black | `#1a1a1a` | Background, dog fur |
| Dark Gray | `#2d2d2d` | Secondary background |
| Cream/Tan | `#e8c07d` | Dog fur highlight |
| Orange-Tan | `#d4956a` | Dog fur midtone |
| Dark Brown | `#8b6914` | Dog fur shadow |
| White | `#ffffff` | Chest marking, highlights |
| Terminal Green | `#4ade80` | Accent, terminal text |
| Coral/Orange | `#ff8c42` | Brand accent (from website) |

---

## Style Guidelines

### DO:
- Keep pixels clean and sharp (no anti-aliasing)
- Use limited color palette (8-16 colors max)
- Maintain the friendly, approachable character
- Include terminal/developer tool visual hints
- Keep the retro 8-bit gaming aesthetic

### DON'T:
- Add gradients or smooth shading
- Use too many colors
- Make the dog look aggressive or scary
- Overcomplicate with too much detail
- Forget the speech bubble element (for larger images)

---

## Post-Processing Recommendations

After generating images with AI:

1. **Clean up pixels** - Remove any anti-aliasing artifacts
2. **Verify dimensions** - Resize to exact required dimensions
3. **Check colors** - Adjust to match brand palette
4. **Test at size** - Verify readability at actual display size
5. **Optimize file size** - Use PNG compression (TinyPNG)

### Tools for Post-Processing:
- **Aseprite** - Professional pixel art editor
- **Piskel** - Free online pixel art tool
- **Photoshop** - With nearest-neighbor scaling
- **GIMP** - Free, with pixel art plugins

---

## Batch Generation Command (Example)

If using CLI tools with Imagen 3 API:

```bash
# Generate OG Image
imagen3 generate \
  --prompt "8-bit pixel art banner, cute black and tan Shiba Inu..." \
  --width 1200 \
  --height 630 \
  --output og-image.png

# Generate Twitter Card
imagen3 generate \
  --prompt "8-bit pixel art Twitter card..." \
  --width 1200 \
  --height 628 \
  --output twitter-card.png
```

---

## Files Checklist

After generation, place files in `/website/public/`:

- [ ] `og-image.png` (1200 x 630)
- [ ] `twitter-card.png` (1200 x 628)
- [ ] `apple-touch-icon.png` (180 x 180)
- [ ] `favicon-32x32.png` (32 x 32)
- [ ] `favicon-16x16.png` (16 x 16)

---

## Testing Social Images

After adding images, test with these validators:

1. **Facebook**: https://developers.facebook.com/tools/debug/
2. **Twitter**: https://cards-dev.twitter.com/validator
3. **LinkedIn**: https://www.linkedin.com/post-inspector/
4. **General**: https://metatags.io/

---

*Document created for Caro.sh SEO image generation*
