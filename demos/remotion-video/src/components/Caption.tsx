import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { colors, fonts } from "../tokens";

type Props = {
  text: string;
  // When the caption appears (local sequence frame).
  startFrame: number;
  // How long the caption stays fully visible (frames). Defaults to "until end".
  durationInFrames?: number;
  // Vertical placement: "top" or "bottom".
  position?: "top" | "bottom";
  // Optional accent color for first / second word emphasis.
  emphasis?: { word: string; color: string };
  fontSize?: number;
};

// On-screen text overlay. Spring-fades-in from below; soft fade-out near end.
export const Caption: React.FC<Props> = ({
  text,
  startFrame,
  durationInFrames,
  position = "bottom",
  emphasis,
  fontSize = 56,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const localFrame = frame - startFrame;
  const inProgress = spring({
    frame: localFrame,
    fps,
    config: { damping: 200 },
    durationInFrames: 12,
  });

  const outProgress =
    durationInFrames !== undefined
      ? interpolate(
          localFrame,
          [durationInFrames - 12, durationInFrames],
          [1, 0],
          { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
        )
      : 1;

  const opacity = Math.max(0, Math.min(1, inProgress * outProgress));
  const y = interpolate(inProgress, [0, 1], [20, 0]);

  const renderText = () => {
    if (!emphasis) return text;
    const parts = text.split(emphasis.word);
    return parts.flatMap((part, i) =>
      i === 0
        ? [<span key={`p${i}`}>{part}</span>]
        : [
            <span key={`e${i}`} style={{ color: emphasis.color }}>
              {emphasis.word}
            </span>,
            <span key={`p${i}`}>{part}</span>,
          ],
    );
  };

  return (
    <div
      style={{
        position: "absolute",
        left: 0,
        right: 0,
        [position]: 96,
        textAlign: "center",
        opacity,
        transform: `translateY(${position === "bottom" ? y : -y}px)`,
        fontFamily: fonts.sans,
        color: colors.textPrimary,
        fontSize,
        fontWeight: 700,
        letterSpacing: -0.5,
        textShadow: "0 4px 24px rgba(0,0,0,0.6)",
      }}
    >
      {renderText()}
    </div>
  );
};
