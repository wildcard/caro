import { AbsoluteFill, Series } from "remotion";
import { colors } from "./tokens";
import { ScenePain } from "./scenes/ScenePain";
import { SceneQueries } from "./scenes/SceneQueries";
import { SceneSafety } from "./scenes/SceneSafety";
import { SceneCloser } from "./scenes/SceneCloser";

// Total: 120 + 420 + 240 + 120 = 900 frames.
export const CaroDemo: React.FC = () => {
  return (
    <AbsoluteFill style={{ backgroundColor: colors.bgDeep }}>
      <Series>
        <Series.Sequence durationInFrames={120}>
          <ScenePain />
        </Series.Sequence>
        <Series.Sequence durationInFrames={420}>
          <SceneQueries />
        </Series.Sequence>
        <Series.Sequence durationInFrames={240}>
          <SceneSafety />
        </Series.Sequence>
        <Series.Sequence durationInFrames={120}>
          <SceneCloser />
        </Series.Sequence>
      </Series>
    </AbsoluteFill>
  );
};
