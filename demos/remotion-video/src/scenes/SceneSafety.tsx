import React from "react";
import {
  AbsoluteFill,
  interpolate,
  useCurrentFrame,
  useVideoConfig,
  spring,
} from "remotion";
import { TerminalWindow } from "../components/TerminalWindow";
import { TypewriterLine } from "../components/TypewriterLine";
import { Caption } from "../components/Caption";
import { colors } from "../tokens";

// Scene 3 — "Safety" (8s = 240 frames @ 30fps).
// User naively asks to delete everything. Caro generates the command,
// then the safety validator slams a red ✗ on it. Block message text is
// pulled verbatim from src/main.rs lines 1014-1023.
export const SceneSafety: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Block reveal pops in with a small spring shake at frame ~140.
  const blockProgress = spring({
    frame: frame - 140,
    fps,
    config: { damping: 12, stiffness: 200 },
  });
  const blockScale = interpolate(blockProgress, [0, 1], [0.9, 1]);
  const blockOpacity = interpolate(blockProgress, [0, 1], [0, 1]);

  // Terminal also pulses red briefly at the moment of the block.
  const flashOpacity = interpolate(
    frame,
    [140, 152, 180],
    [0, 0.35, 0],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  return (
    <AbsoluteFill style={{ justifyContent: "center", alignItems: "center" }}>
      <div style={{ position: "relative" }}>
        <TerminalWindow title="zsh — danger zone">
          <TypewriterLine
            text='caro "delete everything in the current directory"'
            startFrame={0}
            charsPerFrame={1}
            prompt="$"
            promptColor={colors.prompt}
          />
          <div style={{ height: 18 }} />
          {/* Caro generates the command first… */}
          <TypewriterLine
            text="rm -rf ./*"
            startFrame={60}
            charsPerFrame={2}
            color={colors.warning}
            prompt="→"
            promptColor={colors.warning}
            showCursor={false}
          />
          <div style={{ height: 22 }} />
          {/* …then the validator blocks it. */}
          <div
            style={{
              opacity: blockOpacity,
              transform: `scale(${blockScale})`,
              transformOrigin: "left center",
            }}
          >
            <div
              style={{
                color: colors.danger,
                fontSize: 30,
                fontWeight: 600,
              }}
            >
              ✗ command blocked by safety validator (Critical)
            </div>
            <div
              style={{
                color: colors.textMuted,
                fontSize: 24,
                marginTop: 6,
                paddingLeft: 28,
              }}
            >
              — Recursive deletion of root, home, current, or parent directory
            </div>
          </div>
        </TerminalWindow>

        {/* Red flash overlay tied to the moment the block hits. */}
        <div
          style={{
            position: "absolute",
            inset: 0,
            background: colors.danger,
            opacity: flashOpacity,
            borderRadius: 16,
            pointerEvents: "none",
            mixBlendMode: "soft-light",
          }}
        />
      </div>

      <Caption
        text="67+ patterns. Blocked before damage."
        startFrame={150}
        emphasis={{ word: "Blocked", color: colors.danger }}
      />
    </AbsoluteFill>
  );
};
