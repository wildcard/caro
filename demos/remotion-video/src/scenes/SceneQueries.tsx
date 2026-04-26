import React from "react";
import { AbsoluteFill, Series, interpolate, useCurrentFrame } from "remotion";
import { TerminalWindow } from "../components/TerminalWindow";
import { TypewriterLine } from "../components/TypewriterLine";
import { Caption } from "../components/Caption";
import { colors } from "../tokens";

// Scene 2 — "Queries" (14s = 420 frames @ 30fps).
// Three real caro queries from .claude/beta-testing/test-cases.yaml,
// each ~140 frames (4.67s). One terminal, three "exchanges" stacked.

type Query = {
  prompt: string;
  output: string;
  badge: string;
};

const QUERIES: Query[] = [
  {
    prompt: 'caro "find all PDF files larger than 10MB in Downloads"',
    output: 'find ~/Downloads -name "*.pdf" -size +10M -ls',
    badge: "0.3s · 100% local",
  },
  {
    prompt: 'caro "find python files modified last week"',
    output: 'find . -name "*.py" -type f -mtime -7',
    badge: "0.4s · 100% local",
  },
  {
    prompt: 'caro "find all errors in application logs"',
    output: "grep ERROR logs/app.log",
    badge: "0.2s · 100% local",
  },
];

export const SceneQueries: React.FC = () => {
  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <Series>
        {QUERIES.map((q) => (
          <Series.Sequence key={q.prompt} durationInFrames={140}>
            <QueryFrame query={q} />
          </Series.Sequence>
        ))}
      </Series>

      {/* Persistent label across all three exchanges by design.
          Do not add `durationInFrames` here — Caption with no duration
          stays fully visible for the parent sequence's lifetime, which
          is what we want for a scene-level label. */}
      <Caption
        text="One line. Real commands."
        startFrame={0}
        emphasis={{ word: "Real", color: colors.accent }}
      />
    </AbsoluteFill>
  );
};

const QueryFrame: React.FC<{ query: Query }> = ({ query }) => {
  // Frame budget per query: 140
  // 0–55: type the prompt (chars: ~55, 1 char per frame)
  // 60–110: caro thinking → output reveals
  // 110–140: output sits, badge shows
  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <TerminalWindow title="zsh — ~">
        <TypewriterLine
          text={query.prompt}
          startFrame={0}
          charsPerFrame={1}
          prompt="$"
          promptColor={colors.prompt}
        />
        <div style={{ height: 18 }} />
        {/* Output line — typed in caro's accent color, slightly larger. */}
        <TypewriterLine
          text={query.output}
          startFrame={70}
          charsPerFrame={1}
          color={colors.accent}
          prompt="→"
          promptColor={colors.accent}
          showCursor={false}
        />
        <div style={{ height: 22 }} />
        <BadgeLine startFrame={110} text={query.badge} />
      </TerminalWindow>
    </AbsoluteFill>
  );
};

const BadgeLine: React.FC<{ startFrame: number; text: string }> = ({
  startFrame,
  text,
}) => {
  // Reveal as a small, calm pill once the output is done typing.
  return (
    <RevealOnFrame startFrame={startFrame}>
      <div
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: 10,
          fontSize: 22,
          color: colors.textMuted,
          background: colors.accentSoft,
          border: `1px solid ${colors.borderSubtle}`,
          padding: "8px 14px",
          borderRadius: 999,
        }}
      >
        <span
          style={{
            display: "inline-block",
            width: 8,
            height: 8,
            borderRadius: "50%",
            background: colors.success,
          }}
        />
        <span style={{ fontFamily: "inherit" }}>{text}</span>
      </div>
    </RevealOnFrame>
  );
};

const RevealOnFrame: React.FC<{
  startFrame: number;
  children: React.ReactNode;
}> = ({ startFrame, children }) => {
  const frame = useCurrentFrame();
  const opacity = interpolate(
    frame,
    [startFrame, startFrame + 6],
    [0, 1],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );
  return <div style={{ opacity }}>{children}</div>;
};
