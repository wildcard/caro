# Kavana ball interaction research

Status: researched design for a follow-up implementation. This document does not add the ball to the current Kavana contribution.

## Recommendation

Build the ball as an independent interactive actor, not as new cells in Kavana's approved Codex pet atlas. Keep three concerns separate:

1. `KavanaCompanion` owns Kavana's route and gesture state.
2. A `KavanaBall` component owns pointer input, position, velocity, and ball frames.
3. A small coordinator owns the fetch state machine and tells both actors what to do.

This protects the validated 8×11 pet atlas. The ball can be generated and reviewed as its own transparent WebP strip, while Kavana continues using the existing left/right run rows. When she carries it, render the ball as a separate layer at a per-frame muzzle anchor rather than baking it into a second pet atlas.

## Ball asset contract

- File: `website/public/pets/kavana/ball-spritesheet.webp`
- Layout: one horizontal strip, eight 48×48 transparent cells (384×48 total).
- Frames: neutral, four quarter-turns, squash, rebound, and held.
- Style: warm orange-and-cream toy ball with the same pixel scale and outline weight as Kavana.
- Runtime: CSS background-position or the same translated-image technique used by Kavana.
- QA: alpha/background validation, an eight-frame contact sheet, a looping spin preview, and checks at 1×/2× display scale.

The ball art should be generated through the hatch-pet image workflow as a separately grounded prop strip. It must not be appended to `spritesheet.webp` or included in the downloadable two-file Codex pet package.

## Interaction state machine

```text
hidden
  -> offered             visitor activates “Play fetch”
  -> bringing            visitor clicks the offered ball; Kavana runs to it
  -> ready               Kavana returns/places ball in the play area
  -> held                visitor presses and drags ball
  -> flying              visitor releases; velocity comes from recent pointer samples
  -> bouncing            header/floor collision changes velocity
  -> ready               ball settles on the floor
  -> fetching-left       ball crosses the left edge; Kavana runs left
  -> gone-for-session    Kavana and ball remain gone
  -> fetching-right      ball crosses the right edge; Kavana runs right
  -> gone-for-session    Kavana and ball remain gone
```

`gone-for-session` should be stored in `sessionStorage`, so “never return” survives client-side navigation but resets in a new browser session. Provide a visibly labeled “Play fetch” control before the ball appears; the ball itself remains keyboard-focusable and activatable.

## Input and physics

- Use Pointer Events for mouse, touch, and pen through one code path.
- Call `setPointerCapture(pointerId)` on press so a fast throw continues receiving events outside the ball.
- Keep the latest 4–6 timestamped pointer samples. On release, calculate velocity from the recent sample window and cap extreme values.
- Advance physics with `requestAnimationFrame` and its timestamp, using a capped delta to avoid a huge jump after tab suspension.
- Render only with `transform: translate3d(...) rotate(...)`; do not update layout properties every frame.
- Floor: `viewport height - ball diameter - 8px`; clamp there, invert vertical velocity with restitution, apply horizontal friction, and stop below a velocity threshold.
- Header: read the visible header's bottom edge each frame or on resize. Upward throws collide with that plane and bounce downward.
- Left/right: once the ball's center crosses a viewport edge with outward velocity, transition to the matching fetch exit. Do not bounce from side walls.
- Floating controls: use the existing obstacle rectangles so the resting ball cannot settle underneath the mute or back-to-top buttons.

## Kavana coordination

- Pause ordinary roaming whenever fetch is active.
- Choose `runLeft` or `runRight` from the target/ball direction.
- For “bring the ball,” run to the offered ball, attach the independent ball layer to a muzzle anchor, run back to a safe floor position, then drop it.
- Store muzzle anchors as an eight-entry map for each directional run row; this keeps the ball aligned without modifying Kavana's atlas.
- If the ball exits left/right, Kavana runs beyond the same edge and the coordinator enters `gone-for-session`.
- Opening Kavana's dialogue pauses ball physics. Closing resumes only from a safe, stationary state.

## Reduced motion and accessibility

- Under `prefers-reduced-motion: reduce`, disable autonomous ball bobbing/spinning and Kavana's run-frame loop.
- Keep dragging functional. On release, move the ball directly to the predicted landing point or use one short, non-bouncy transition.
- Do not make fetch the only way to access any content.
- Give the ball a real button label such as “Kavana's ball — drag or press to play.” Announce state changes (`Kavana brought the ball`, `The ball bounced off the header`, `Kavana ran off to fetch it`) in a polite live region.
- Provide a keyboard throw: arrow keys choose direction/power, Space releases, and Escape cancels a held ball.

## Verification gates

- Pointer capture works when the pointer leaves the ball and viewport.
- Header collision always reverses upward velocity; the ball never renders above the header.
- Floor collision always contains the ball; it settles without jitter.
- Left/right exit makes both actors leave and stay gone for the browser session.
- Click, drag, dialogue, Kavana placement, sleep, and floating controls do not steal one another's pointer events.
- Mobile touch, viewport resize, dark mode, and reduced motion each have a recorded browser test.
- The independent ball strip passes visual/alpha QA and the original Kavana atlas and ZIP hashes remain unchanged.

## Sources

- [MDN: Pointer events](https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events) — unified mouse/touch/pen events, cancellation, and capture behavior.
- [MDN: `setPointerCapture()`](https://developer.mozilla.org/en-US/docs/Web/API/Element/setPointerCapture) — retaining pointer events during an out-of-bounds drag.
- [MDN: `requestAnimationFrame()`](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame) — repaint scheduling and timestamp-based animation.
- [MDN: CSS and JavaScript animation performance](https://developer.mozilla.org/en-US/docs/Web/Performance/Guides/CSS_JavaScript_animation_performance) — efficient JavaScript-driven animation.
- [web.dev: Animations and performance](https://web.dev/articles/animations-and-performance) — transform/opacity-focused rendering guidance.
