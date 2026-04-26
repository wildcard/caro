import { Composition } from "remotion";
import { CaroDemo } from "./CaroDemo";

// 30 seconds @ 30fps = 900 frames.
// 1920x1080 — retina-friendly hero asset.
export const Root: React.FC = () => {
  return (
    <Composition
      id="CaroDemo"
      component={CaroDemo}
      durationInFrames={900}
      fps={30}
      width={1920}
      height={1080}
    />
  );
};
