import React from "react";
import {
  AbsoluteFill,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { colors, fonts } from "../tokens";

// Scene 4 — "Closer" (4s = 120 frames @ 30fps).
// Centered logotype + tagline + install line. No terminal here — this
// is the visual exhale.
export const SceneCloser: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoSpring = spring({
    frame,
    fps,
    config: { damping: 14, stiffness: 120 },
  });
  const logoScale = interpolate(logoSpring, [0, 1], [0.85, 1]);
  const logoOpacity = interpolate(frame, [0, 12], [0, 1], {
    extrapolateRight: "clamp",
  });

  const taglineOpacity = interpolate(frame, [22, 36], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const installOpacity = interpolate(frame, [44, 60], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const installY = interpolate(frame, [44, 60], [12, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",
        background: `radial-gradient(ellipse at center, ${colors.bgTerminal} 0%, ${colors.bgDeep} 70%)`,
      }}
    >
      {/* Logotype */}
      <div
        style={{
          opacity: logoOpacity,
          transform: `scale(${logoScale})`,
          fontFamily: fonts.sans,
          fontSize: 144,
          fontWeight: 800,
          letterSpacing: -4,
          color: colors.textPrimary,
          marginBottom: 20,
        }}
      >
        <span style={{ color: colors.accent }}>caro</span>
        <span style={{ color: colors.textDim, fontWeight: 400 }}>.sh</span>
      </div>

      {/* Tagline */}
      <div
        style={{
          opacity: taglineOpacity,
          fontFamily: fonts.sans,
          fontSize: 38,
          fontWeight: 600,
          color: colors.textPrimary,
          marginBottom: 60,
          textAlign: "center",
        }}
      >
        Local. Private. <span style={{ color: colors.accent }}>No API key.</span>
      </div>

      {/* Install line */}
      <div
        style={{
          opacity: installOpacity,
          transform: `translateY(${installY}px)`,
          display: "inline-flex",
          alignItems: "center",
          gap: 16,
          padding: "20px 32px",
          background: colors.bgChrome,
          border: `1px solid ${colors.borderSubtle}`,
          borderRadius: 12,
          fontFamily: fonts.mono,
          fontSize: 32,
          color: colors.textPrimary,
        }}
      >
        <span style={{ color: colors.prompt }}>$</span>
        <span>cargo install caro</span>
      </div>

      {/* Subtle URL footer */}
      <div
        style={{
          position: "absolute",
          bottom: 64,
          opacity: installOpacity,
          fontFamily: fonts.sans,
          fontSize: 24,
          color: colors.textMuted,
          letterSpacing: 1,
        }}
      >
        caro.sh
      </div>
    </AbsoluteFill>
  );
};
