import React from "react";
import { interpolate, useCurrentFrame } from "remotion";
import { colors } from "../tokens";

type Props = {
  text: string;
  // Frame at which typing begins (within parent sequence).
  startFrame: number;
  // Frames per character. ~3 = brisk; ~6 = relaxed.
  charsPerFrame?: number;
  // Show a blinking cursor at the end while typing and after.
  showCursor?: boolean;
  color?: string;
  // Optional prefix (e.g. "$ ") rendered without typing animation.
  prompt?: string;
  promptColor?: string;
  fontSize?: number;
};

// Reveals `text` character by character based on the current frame.
// `useCurrentFrame()` inside a <Sequence> is local (starts at 0), so callers
// pass startFrame in the local sequence frame space.
export const TypewriterLine: React.FC<Props> = ({
  text,
  startFrame,
  charsPerFrame = 3,
  showCursor = true,
  color = colors.command,
  prompt,
  promptColor = colors.prompt,
  fontSize = 30,
}) => {
  const frame = useCurrentFrame();

  const charsToShow = Math.max(
    0,
    Math.min(
      text.length,
      Math.floor((frame - startFrame) / Math.max(1, charsPerFrame)),
    ),
  );

  const visibleText = text.slice(0, charsToShow);
  const finishedTyping = charsToShow >= text.length;

  // Cursor blinks every 0.5s (15 frames @ 30fps).
  const cursorOpacity = Math.floor(frame / 15) % 2 === 0 ? 1 : 0;

  // Subtle fade-in of the whole line as typing starts.
  const lineOpacity = interpolate(
    frame,
    [startFrame - 4, startFrame],
    [0, 1],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  return (
    <div style={{ opacity: lineOpacity, fontSize, lineHeight: 1.4 }}>
      {prompt ? (
        <span style={{ color: promptColor, marginRight: 12 }}>{prompt}</span>
      ) : null}
      <span style={{ color }}>{visibleText}</span>
      {showCursor && frame >= startFrame ? (
        <span
          style={{
            display: "inline-block",
            width: "0.55em",
            height: "1em",
            verticalAlign: "text-bottom",
            marginLeft: 2,
            background: color,
            opacity: finishedTyping ? cursorOpacity : 1,
          }}
        />
      ) : null}
    </div>
  );
};
