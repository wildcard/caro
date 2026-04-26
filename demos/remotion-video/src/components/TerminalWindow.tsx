import React from "react";
import { colors, fonts } from "../tokens";

type Props = {
  title?: string;
  children: React.ReactNode;
  width?: number;
  // For drop-shadow elevation; defaults read fine on the dark bg.
  elevated?: boolean;
};

// macOS-style window chrome — matches website/src/components/landing/LPDemo.astro
// so the rendered video reads as "the same terminal you see on the site".
export const TerminalWindow: React.FC<Props> = ({
  title = "caro",
  children,
  width = 1280,
  elevated = true,
}) => {
  return (
    <div
      style={{
        width,
        borderRadius: 16,
        overflow: "hidden",
        border: `1px solid ${colors.borderSubtle}`,
        background: colors.bgTerminal,
        boxShadow: elevated
          ? "0 30px 80px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255,255,255,0.02)"
          : "none",
        fontFamily: fonts.mono,
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          padding: "14px 16px",
          background: colors.bgChrome,
          borderBottom: `1px solid ${colors.borderSubtle}`,
        }}
      >
        <Dot color={colors.trafficClose} />
        <Dot color={colors.trafficMin} />
        <Dot color={colors.trafficMax} />
        <div
          style={{
            flex: 1,
            textAlign: "center",
            color: colors.textDim,
            fontSize: 14,
            fontFamily: fonts.sans,
            letterSpacing: 0.2,
          }}
        >
          {title}
        </div>
        {/* Right-side spacer to balance the dots */}
        <div style={{ width: 52 }} />
      </div>
      <div
        style={{
          padding: "32px 36px",
          minHeight: 360,
          color: colors.textPrimary,
          fontSize: 28,
          lineHeight: 1.55,
        }}
      >
        {children}
      </div>
    </div>
  );
};

const Dot: React.FC<{ color: string }> = ({ color }) => (
  <div
    style={{
      width: 14,
      height: 14,
      borderRadius: "50%",
      background: color,
    }}
  />
);
