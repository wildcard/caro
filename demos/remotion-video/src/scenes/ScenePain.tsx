import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { TerminalWindow } from "../components/TerminalWindow";
import { TypewriterLine } from "../components/TypewriterLine";
import { Caption } from "../components/Caption";
import { colors } from "../tokens";

// Scene 1 — "Pain" (4s = 120 frames @ 30fps).
// User starts typing a comment in the terminal, trails off uncertainly.
// Caption hammers the relatable emotion.
export const ScenePain: React.FC = () => {
  const frame = useCurrentFrame();

  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <TerminalWindow title="zsh — ~/Downloads">
        {/* Welcome line — appears immediately. */}
        <div style={{ color: colors.textMuted, marginBottom: 18 }}>
          Last login: today at the wrong moment
        </div>
        {/* User starts to write a comment to themselves and gives up. */}
        <TypewriterLine
          text="# how do I find all the PDFs over 10 megabytes again..."
          startFrame={10}
          charsPerFrame={2}
          color={colors.textMuted}
          prompt="$"
          promptColor={colors.prompt}
        />
        {/* After typing finishes (~120 frames in), keep cursor blinking. */}
        {frame > 90 ? (
          <div style={{ marginTop: 12, color: colors.textDim, fontSize: 24 }}>
            ↑ <em>(opens browser tab to Stack Overflow…)</em>
          </div>
        ) : null}
      </TerminalWindow>

      <Caption
        text="Forgot the syntax. Again."
        startFrame={30}
        emphasis={{ word: "Again.", color: colors.accent }}
      />
    </AbsoluteFill>
  );
};
