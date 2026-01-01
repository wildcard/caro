# Kyaro Sprite Sheet Guide

> **Comprehensive guide for Aci (Art Director) to design Kyaro, the cmdai mascot**

---

## 🐕 Who is Kyaro?

**Kyaro** is cmdai's loyal Shiba-inspired terminal companion. She's:
- A Pokémon-like character (think Growlithe meets Eevee)
- Expressive, reactive, and warm
- Your buddy in the terminal
- The emotional anchor of the brand

---

## 🎨 Design Direction

### Core Characteristics

**Species:** Shiba Inu inspired
**Style:** 8-bit/16-bit pixel art (Game Boy era)
**Personality:** Loyal, friendly, slightly sassy, protective
**Age:** Young adult (not puppy, not old)
**Energy:** Medium-high (alert but calm)

### Visual References

**Pokémon Inspirations:**
- **Growlithe** - loyal fire dog
- **Eevee** - cute, expressive
- **Vulpix** - fox-like charm
- **Rockruff** - friendly dog

**Shiba Inu Traits:**
- Curled tail
- Pointed ears
- Fox-like face
- Cream/tan markings
- Alert expression

### Color Palette for Kyaro

**Primary colors:**
```
Body (cream):     #f4d5a6  (warm cream)
Markings (tan):   #d4a574  (golden tan)
Face (white):     #fff5e6  (off-white)
Nose:             #2d2d2d  (dark gray)
Eyes:             #3d2d1f  (dark brown)
Accent:           #ff6b35  (warm orange - collar/accessories)
```

**Shading (for larger sprites):**
```
Light:            #fef0d8
Mid:              #f4d5a6
Shadow:           #c9a876
```

---

## 📐 Sprite Specifications

### Size Options

**Terminal (ANSI art):**
- 16x16 pixels (tiny, for compact terminals)
- 32x32 pixels (standard, most common)

**Web (Hero section):**
- 64x64 pixels (retro but detailed)
- 128x128 pixels (large hero image)
- 256x256 pixels (maximum detail)

### File Formats

- **PNG** with transparency (alpha channel)
- **8-bit indexed color** (256 colors max)
- **Sprite sheets** (all states in one image)
- **Individual frames** (for animation)

### Naming Convention

```
kyaro-[state]-[size].png

Examples:
kyaro-idle-32.png
kyaro-thinking-64.png
kyaro-success-128.png
```

---

## 🎭 11 Essential States

### 1. Idle
**Emotion:** Neutral, resting
**Use Case:** Default state when nothing is happening
**Design Notes:**
- Sitting position
- Ears slightly forward
- Tail curled comfortably
- Blinking animation (slow)
- Subtle breathing motion

### 2. Waiting
**Emotion:** Anticipating, ready
**Use Case:** User is about to type a command
**Design Notes:**
- Sitting upright
- Ears perked up
- Tail slightly raised
- Eyes focused
- Small head tilt

### 3. Listening
**Emotion:** Actively processing
**Use Case:** Command being parsed/analyzed
**Design Notes:**
- Ears fully forward
- Slight lean forward
- Tail straight
- Eyes wide, attentive
- Possible sound wave icon nearby

### 4. Thinking
**Emotion:** Deep concentration
**Use Case:** LLM inference, complex processing
**Design Notes:**
- Sitting, head slightly down
- Paw to chin (classic thinking pose)
- Tail relaxed
- Sweat drop or thought bubble
- Loading indicator (spinning, dots)

### 5. Success
**Emotion:** Happy, celebrating
**Use Case:** Command executed successfully
**Design Notes:**
- Jumping or standing on hind legs
- Tail wagging (motion lines)
- Happy eyes (^_^)
- Possible sparkles or stars
- Tongue out (playful)

### 6. Warning
**Emotion:** Cautious, alert
**Use Case:** Potentially dangerous command detected
**Design Notes:**
- Standing, protective stance
- Ears back slightly
- Tail down but alert
- Serious eyes
- Exclamation mark (!) nearby
- Yellow/amber glow

### 7. Error
**Emotion:** Sad, concerned
**Use Case:** Command failed or blocked
**Design Notes:**
- Sitting, ears down
- Tail lowered
- Sad eyes (T_T or ;_;)
- Small X mark
- Red glow or broken heart icon

### 8. Bored
**Emotion:** Sleepy, unengaged
**Use Case:** No user activity for a while
**Design Notes:**
- Lying down or slouching
- Eyes half-closed
- Tail limp
- Zzz symbols
- Yawning animation (optional)

### 9. Long-inference
**Emotion:** Patient waiting
**Use Case:** Model taking a long time
**Design Notes:**
- Sitting, fidgeting slightly
- Looking at a clock or hourglass
- Tail swishing slowly
- Eyes wandering
- Thought bubble: "..."

### 10. Greeting
**Emotion:** Welcoming, friendly
**Use Case:** First launch, hello message
**Design Notes:**
- Standing, front paw raised (waving)
- Tail wagging
- Happy eyes
- Mouth open (smiling)
- Speech bubble: "Hello!" or "Woof!"

### 11. Farewell
**Emotion:** Waving goodbye
**Use Case:** Exit, shutdown
**Design Notes:**
- Standing, paw raised (waving)
- Tail wagging gently
- Gentle smile
- Speech bubble: "Bye!" or "See you!"
- Fade-out animation (optional)

---

## 🎬 Animation Guidelines

### Frame Timing

**Idle animations:**
- 2-4 frames
- 500-1000ms per frame
- Loop infinitely

**Action animations:**
- 4-8 frames
- 100-200ms per frame
- Play once or loop 2-3 times

**Transition animations:**
- 2-3 frames
- 150-300ms per frame
- Play once

### Animation Principles

- **Keep it simple** - 8-bit style, minimal frames
- **Readable motion** - clear, not blurry
- **Personality** - each animation shows character
- **Terminal-friendly** - works in ANSI art

### Example Animation Sequence

**Success (Jump for joy):**
```
Frame 1: Crouched (preparing to jump)
Frame 2: Mid-air (legs tucked)
Frame 3: Peak (fully extended)
Frame 4: Landing (paws hitting ground)
Frame 5: Settle (back to standing)
Frame 6: Tail wag (left)
Frame 7: Tail wag (right)
Frame 8: Return to idle
```

---

## 🖼️ Sprite Sheet Layout

### Standard Layout (32x32 sprites)

```
┌──────┬──────┬──────┬──────┐
│ Idle │ Wait │Listen│Think │
├──────┼──────┼──────┼──────┤
│Success│Warn │Error │Bored │
├──────┼──────┼──────┼──────┤
│Long-I│Greet │ Bye  │ --- │
└──────┴──────┴──────┴──────┘

Each cell: 32x32 pixels
Total sheet: 128x96 pixels (4x3 grid)
```

### Animation Sheet (4 frames each)

```
┌──┬──┬──┬──┐  ┌──┬──┬──┬──┐
│    Idle    │  │   Success  │
├──┬──┬──┬──┤  ├──┬──┬──┬──┤
│  Thinking  │  │   Warning  │
└──┴──┴──┴──┘  └──┴──┴──┴──┘
```

---

## 🎨 ASCII Fallback

### Requirements

**For limited terminals that can't display pixel art:**
- Must be recognizable as Kyaro
- Works in pure ASCII (no ANSI colors)
- Fits in ~7 lines
- Clear silhouette

### ASCII Kyaro Options

**Option 1: Side view (detailed)**
```
    /\_/\
   ( o.o )
    > ^ <
   /|   |\
  (_|   |_)
  ~~~ ~~~
```

**Option 2: Front view (simple)**
```
   /\_/\
  ( ^.^ )
   >   <
  /|___|\
```

**Option 3: Minimal**
```
  (\/)
  (oo)
  (/\)
```

**Recommendation:** Use Option 1 for standard, Option 3 for compact

### ASCII States

**Each state needs ASCII variant:**

```
Idle:     ( o.o )
Thinking: ( -.- )
Success:  ( ^.^ )
Warning:  ( O.O )
Error:    ( ;_; )
Bored:    ( u.u )
```

---

## 🎯 Deliverables Checklist

### Phase 1: Concept (Week 1)
- [ ] Kyaro reference sheet (front, side, back)
- [ ] Color palette defined
- [ ] 2-3 pose sketches
- [ ] Personality notes documented

### Phase 2: Core States (Week 2)
- [ ] Idle state (32x32, 64x64)
- [ ] Thinking state
- [ ] Success state
- [ ] Error state
- [ ] Warning state

### Phase 3: Extended States (Week 3)
- [ ] Waiting state
- [ ] Listening state
- [ ] Bored state
- [ ] Long-inference state
- [ ] Greeting state
- [ ] Farewell state

### Phase 4: Animations (Week 4)
- [ ] Idle animation (2-4 frames)
- [ ] Success animation (4-8 frames)
- [ ] Thinking animation (2-4 frames)
- [ ] Sprite sheets compiled

### Phase 5: ASCII & Polish (Week 5)
- [ ] ASCII fallback designed
- [ ] All sizes exported (16, 32, 64, 128, 256)
- [ ] Documentation complete
- [ ] Integration guide for Sulo

---

## 📏 Technical Specifications

### Pixel Art Rules

**Do:**
- ✅ Use limited color palette (8-16 colors)
- ✅ Crisp, clean edges (no anti-aliasing)
- ✅ Consistent pixel size
- ✅ Readable at small sizes
- ✅ High contrast for visibility

**Don't:**
- ❌ Use gradients (hard in pixel art)
- ❌ Anti-alias edges (blurry pixels)
- ❌ Too many colors (keep it simple)
- ❌ Blur or smooth (defeats pixel aesthetic)

### Export Settings

**For Aseprite:**
```
File → Export Sprite Sheet
- Layout: By Rows
- Padding: 1 pixel
- Output: PNG-8 (indexed)
- Scale: 100% (no scaling)
```

**For Web:**
```css
image-rendering: pixelated;
-ms-interpolation-mode: nearest-neighbor;
```

---

## 🔗 Integration Notes for Sulo

### How to Use Sprites

**In React components:**
```tsx
import Image from 'next/image';

<Image
  src="/mascot/kyaro-idle-64.png"
  alt="Kyaro - cmdai mascot"
  width={64}
  height={64}
  className="sprite-animate"
  style={{ imageRendering: 'pixelated' }}
/>
```

**Animation with sprite sheet:**
```css
.kyaro-sprite {
  width: 32px;
  height: 32px;
  background: url('/mascot/kyaro-sheet-32.png') no-repeat;
  image-rendering: pixelated;
  animation: kyaro-idle 1s steps(4) infinite;
}

@keyframes kyaro-idle {
  0% { background-position: 0 0; }
  25% { background-position: -32px 0; }
  50% { background-position: -64px 0; }
  75% { background-position: -96px 0; }
}
```

### File Organization

```
apps/devrel/public/mascot/
├── kyaro-reference.png          # Reference sheet
├── kyaro-idle-32.png            # Individual states
├── kyaro-thinking-32.png
├── kyaro-success-32.png
├── ...
├── sheets/
│   ├── kyaro-all-32.png         # Complete sprite sheet
│   ├── kyaro-animations-32.png  # Animation frames
│   └── kyaro-ascii.txt          # ASCII variants
└── source/
    └── kyaro.aseprite           # Source file (if using Aseprite)
```

---

## 💡 Creative Tips for Aci

### Shiba Personality Traits to Capture

- **Loyal but independent** - not overly clingy
- **Alert** - ears always active
- **Playful** - enjoys interaction
- **Dignified** - maintains composure
- **Expressive** - emotions are clear

### Inspiration Sources

**Pokémon Sprites:**
- Study Gen 1-2 Pokémon sprites (Game Boy era)
- Note how personality shows in limited pixels
- Look at Growlithe, Eevee, Vulpix animations

**Real Shibas:**
- Reference real Shiba Inu photos
- Note the curled tail (signature feature)
- Study ear positions (very expressive)
- Observe sitting/standing poses

**Classic Pixel Art:**
- EarthBound characters (personality + simple)
- Undertale sprites (expressive with few pixels)
- Stardew Valley animals (cute + functional)

### Design Process

1. **Sketch rough concepts** (paper or digital)
2. **Block out basic shape** (silhouette test)
3. **Add key features** (ears, tail, face)
4. **Refine details** (eyes, nose, markings)
5. **Test at target size** (does it read clearly?)
6. **Create variations** (different states)
7. **Animate** (bring to life)

---

## ✨ Success Criteria

**Kyaro is ready when:**
- [ ] Instantly recognizable as a Shiba
- [ ] Personality shines through in each state
- [ ] Works at all target sizes (16-256px)
- [ ] ASCII fallback is clear and cute
- [ ] Animations are smooth and charming
- [ ] Colors match brand palette
- [ ] Files are properly organized
- [ ] Documentation is complete

---

## 📞 Questions for Aci?

If you need clarification on:
- **Design direction:** Check BRAND_IDENTITY.md
- **Technical specs:** Check this file
- **Integration:** Ask Sulo
- **Brand alignment:** Discuss with team

**Slack:** #devrel-website
**Tag:** @aci or @team

---

**Let's bring Kyaro to life! 🐕✨**
