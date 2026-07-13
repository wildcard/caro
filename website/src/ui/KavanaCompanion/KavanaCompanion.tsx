import { useEffect, useMemo, useRef, useState, type CSSProperties } from 'react';
import styles from './KavanaCompanion.module.css';

type Topic = 'welcome' | 'status' | 'adopt' | 'hatch';
type AnimationName = 'idle' | 'runRight' | 'runLeft' | 'wave' | 'jump' | 'waiting' | 'resting' | 'working' | 'review' | 'lookAround';

interface SpriteFrame {
  row: number;
  column: number;
  duration: number;
}

interface RoamStep {
  id: string;
  x: string;
  y: string;
  animation: AnimationName;
  duration: number;
  travelTime: number;
  side: 'left' | 'right' | 'center';
}

interface Position {
  x: number;
  y: number;
}

interface DragSession extends Position {
  pointerId: number;
  startX: number;
  startY: number;
  lastX: number;
  moved: boolean;
}

export interface KavanaCompanionProps {
  embedded?: boolean;
  initiallyOpen?: boolean;
}

const topics: Record<Topic, { eyebrow: string; title: string; body: string }> = {
  welcome: { eyebrow: 'Your project companion', title: "Hi, I'm Kavana.", body: 'My name means intention. I keep an eye on Caro while the humans build a safer, friendlier way to turn plain language into shell commands.' },
  status: { eyebrow: 'Development pulse', title: 'Caro is actively growing.', body: 'The public roadmap tracks work across core reliability, documentation, releases, and local-model support. Follow the evidence—not a mystery progress bar.' },
  adopt: { eyebrow: 'Bring me home', title: 'I can join your Codex app.', body: 'Install the official Codex desktop app first, then download my community pet package and follow the two-file setup guide.' },
  hatch: { eyebrow: 'Make a companion', title: 'Or hatch your own pet.', body: 'Open a Codex task with your character idea or reference art and ask Codex to use the hatch-pet skill. It can assemble and validate the complete animated atlas.' },
};

const rowFrames = (row: number, durations: number[]): SpriteFrame[] => durations.map((duration, column) => ({ row, column, duration }));

const animations: Record<AnimationName, SpriteFrame[]> = {
  idle: rowFrames(0, [280, 110, 110, 140, 140, 320]),
  runRight: rowFrames(1, [120, 120, 120, 120, 120, 120, 120, 220]),
  runLeft: rowFrames(2, [120, 120, 120, 120, 120, 120, 120, 220]),
  wave: rowFrames(3, [140, 140, 140, 280]),
  jump: rowFrames(4, [140, 140, 140, 140, 280]),
  waiting: rowFrames(6, [150, 150, 150, 150, 150, 260]),
  resting: [
    { row: 6, column: 2, duration: 650 },
    { row: 6, column: 3, duration: 220 },
    { row: 6, column: 4, duration: 650 },
    { row: 6, column: 3, duration: 220 },
  ],
  working: rowFrames(7, [120, 120, 120, 120, 120, 220]),
  review: rowFrames(8, [150, 150, 150, 150, 150, 280]),
  lookAround: [
    ...rowFrames(9, [190, 150, 150, 150, 190, 150, 150, 150]),
    ...rowFrames(10, [190, 150, 150, 150, 190, 150, 150, 240]),
  ],
};

const entranceStart: RoamStep = {
  id: 'offstage', x: '-150px', y: 'calc(100vh - 150px)', animation: 'runRight', duration: 0, travelTime: 0, side: 'left',
};

const roamRoute: RoamStep[] = [
  { id: 'entrance', x: '22px', y: 'calc(100vh - 150px)', animation: 'runRight', duration: 3000, travelTime: 2500, side: 'left' },
  { id: 'hello', x: '22px', y: 'calc(100vh - 150px)', animation: 'wave', duration: 1800, travelTime: 0, side: 'left' },
  { id: 'bottom-cross', x: 'calc(100vw - 220px)', y: 'calc(100vh - 150px)', animation: 'runRight', duration: 5700, travelTime: 5100, side: 'right' },
  { id: 'scan-right', x: 'calc(100vw - 220px)', y: 'calc(100vh - 150px)', animation: 'lookAround', duration: 3100, travelTime: 0, side: 'right' },
  { id: 'hop-up', x: 'calc(100vw - 170px)', y: '42vh', animation: 'jump', duration: 2600, travelTime: 2100, side: 'right' },
  { id: 'review', x: 'calc(100vw - 170px)', y: '42vh', animation: 'review', duration: 2600, travelTime: 0, side: 'right' },
  { id: 'upper-cross', x: '30px', y: '48vh', animation: 'runLeft', duration: 6400, travelTime: 5800, side: 'left' },
  { id: 'waiting', x: '30px', y: '48vh', animation: 'waiting', duration: 2600, travelTime: 0, side: 'left' },
  { id: 'hop-home', x: 'calc(50vw - 48px)', y: 'calc(100vh - 170px)', animation: 'jump', duration: 3000, travelTime: 2400, side: 'center' },
  { id: 'working', x: 'calc(50vw - 48px)', y: 'calc(100vh - 170px)', animation: 'working', duration: 2800, travelTime: 0, side: 'center' },
  { id: 'return-right', x: 'calc(100vw - 220px)', y: 'calc(100vh - 150px)', animation: 'runRight', duration: 5000, travelTime: 4400, side: 'right' },
  { id: 'curious', x: 'calc(100vw - 220px)', y: 'calc(100vh - 150px)', animation: 'lookAround', duration: 3100, travelTime: 0, side: 'right' },
  { id: 'bottom-left', x: '22px', y: 'calc(100vh - 150px)', animation: 'runLeft', duration: 6100, travelTime: 5500, side: 'left' },
  { id: 'welcome-back', x: '22px', y: 'calc(100vh - 150px)', animation: 'wave', duration: 1800, travelTime: 0, side: 'left' },
];

const roamingHints = ['Psst—need a guide?', 'Roadmap check-in ready', 'Want a Codex pet?', 'Following the work…'];

export function KavanaCompanion({ embedded = false, initiallyOpen = false }: KavanaCompanionProps) {
  const [open, setOpen] = useState(initiallyOpen);
  const [topic, setTopic] = useState<Topic>('welcome');
  const [routeIndex, setRouteIndex] = useState(-1);
  const [frameIndex, setFrameIndex] = useState(0);
  const [copied, setCopied] = useState(false);
  const [copyFailed, setCopyFailed] = useState(false);
  const [hintIndex, setHintIndex] = useState(0);
  const [reducedMotion, setReducedMotion] = useState(false);
  const [hovered, setHovered] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [dragDirection, setDragDirection] = useState<'left' | 'right'>('right');
  const [dragPosition, setDragPosition] = useState<Position>();
  const [placedPosition, setPlacedPosition] = useState<Position>();
  const copyFeedbackTimeout = useRef<number>();
  const dragSession = useRef<DragSession>();
  const suppressClick = useRef(false);

  useEffect(() => {
    const preference = window.matchMedia('(prefers-reduced-motion: reduce)');
    const updatePreference = () => setReducedMotion(preference.matches);
    updatePreference();
    if ('addEventListener' in preference) {
      preference.addEventListener('change', updatePreference);
      return () => preference.removeEventListener('change', updatePreference);
    }
    preference.addListener(updatePreference);
    return () => preference.removeListener(updatePreference);
  }, []);

  useEffect(() => () => window.clearTimeout(copyFeedbackTimeout.current), []);

  useEffect(() => {
    if (embedded || open || reducedMotion || dragging || placedPosition) return;
    if (routeIndex < 0) {
      const entrance = window.setTimeout(() => setRouteIndex(0), 120);
      return () => window.clearTimeout(entrance);
    }
    const step = roamRoute[routeIndex];
    const advance = window.setTimeout(() => {
      setRouteIndex((value) => value >= roamRoute.length - 1 ? 2 : value + 1);
    }, step.duration);
    return () => window.clearTimeout(advance);
  }, [dragging, embedded, open, placedPosition, reducedMotion, routeIndex]);

  useEffect(() => {
    if (embedded || open) return;
    const hints = window.setInterval(() => setHintIndex((value) => (value + 1) % roamingHints.length), 6000);
    return () => window.clearInterval(hints);
  }, [embedded, open]);

  const routeStep = reducedMotion ? roamRoute[0] : routeIndex < 0 ? entranceStart : roamRoute[routeIndex];
  const restingStep: RoamStep | undefined = placedPosition ? {
    id: 'placed', x: `${placedPosition.x}px`, y: `${placedPosition.y}px`, animation: 'resting', duration: 0, travelTime: 0,
    side: placedPosition.x > window.innerWidth / 2 ? 'right' : 'left',
  } : undefined;
  const dragStep: RoamStep | undefined = dragPosition ? {
    id: 'dragging', x: `${dragPosition.x}px`, y: `${dragPosition.y}px`, animation: dragDirection === 'right' ? 'runRight' : 'runLeft', duration: 0, travelTime: 0,
    side: dragPosition.x > window.innerWidth / 2 ? 'right' : 'left',
  } : undefined;
  const step = dragStep ?? restingStep ?? routeStep;
  const animationName: AnimationName = dragging ? step.animation : open ? 'idle' : placedPosition ? 'resting' : hovered ? 'wave' : embedded ? 'waiting' : step.animation;
  const animation = animations[animationName];

  useEffect(() => setFrameIndex(0), [animationName]);

  useEffect(() => {
    if (reducedMotion) {
      setFrameIndex(0);
      return;
    }
    const frame = animation[frameIndex % animation.length];
    const advance = window.setTimeout(() => setFrameIndex((value) => (value + 1) % animation.length), frame.duration);
    return () => window.clearTimeout(advance);
  }, [animation, frameIndex, reducedMotion]);

  const frame = animation[frameIndex % animation.length];
  const content = topics[topic];
  const atRight = !embedded && step.side === 'right';
  const motionStyle = useMemo(() => ({
    '--kavana-x': step.x,
    '--kavana-y': step.y,
    '--kavana-travel-time': reducedMotion ? '0ms' : `${step.travelTime}ms`,
  }) as CSSProperties, [reducedMotion, step.travelTime, step.x, step.y]);

  const copyHatchPrompt = async () => {
    window.clearTimeout(copyFeedbackTimeout.current);
    try {
      await navigator.clipboard.writeText('Use the hatch-pet skill to create a Codex pet from my idea or attached reference art. Package and validate the full v2 animated spritesheet, and show me the final QA preview.');
      setCopyFailed(false);
      setCopied(true);
      copyFeedbackTimeout.current = window.setTimeout(() => setCopied(false), 1800);
    } catch {
      setCopied(false);
      setCopyFailed(true);
      copyFeedbackTimeout.current = window.setTimeout(() => setCopyFailed(false), 3000);
    }
  };

  const closeCompanion = () => {
    setOpen(false);
    if (!embedded && !reducedMotion) setRouteIndex(3);
  };

  const toggleOpen = () => open ? closeCompanion() : setOpen(true);

  const avoidFloatingControls = ({ x, y }: Position): Position => {
    const petWidth = 96;
    const petHeight = 120;
    const padding = 12;
    let safe = {
      x: Math.min(Math.max(8, x), window.innerWidth - petWidth - 8),
      y: Math.min(Math.max(8, y), window.innerHeight - petHeight - 8),
    };

    for (const selector of ['#mute-toggle', '#back-to-top']) {
      const obstacle = document.querySelector<HTMLElement>(selector);
      if (!obstacle) continue;
      const bounds = obstacle.getBoundingClientRect();
      const overlaps = safe.x < bounds.right + padding && safe.x + petWidth > bounds.left - padding
        && safe.y < bounds.bottom + padding && safe.y + petHeight > bounds.top - padding;
      if (!overlaps) continue;

      const leftOfControl = bounds.left - petWidth - padding;
      const rightOfControl = bounds.right + padding;
      safe.x = leftOfControl >= 8 ? leftOfControl : Math.min(rightOfControl, window.innerWidth - petWidth - 8);
    }
    return safe;
  };

  const beginDrag = (event: React.PointerEvent<HTMLButtonElement>) => {
    if (embedded || open || event.button !== 0) return;
    const bounds = event.currentTarget.closest('aside')?.getBoundingClientRect();
    if (!bounds) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    dragSession.current = {
      pointerId: event.pointerId,
      x: event.clientX - bounds.left,
      y: event.clientY - bounds.top,
      startX: event.clientX,
      startY: event.clientY,
      lastX: event.clientX,
      moved: false,
    };
    setDragging(true);
    setDragPosition({ x: bounds.left, y: bounds.top });
    setHovered(false);
  };

  const moveDrag = (event: React.PointerEvent<HTMLButtonElement>) => {
    const session = dragSession.current;
    if (!session || session.pointerId !== event.pointerId) return;
    const deltaX = event.clientX - session.lastX;
    if (Math.abs(deltaX) > 1) setDragDirection(deltaX > 0 ? 'right' : 'left');
    session.lastX = event.clientX;
    session.moved ||= Math.hypot(event.clientX - session.startX, event.clientY - session.startY) > 6;
    setDragPosition({
      x: Math.min(Math.max(8, event.clientX - session.x), window.innerWidth - 104),
      y: Math.min(Math.max(8, event.clientY - session.y), window.innerHeight - 128),
    });
  };

  const endDrag = (event: React.PointerEvent<HTMLButtonElement>) => {
    const session = dragSession.current;
    if (!session || session.pointerId !== event.pointerId) return;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
    if (session.moved && dragPosition) {
      setPlacedPosition(avoidFloatingControls(dragPosition));
      suppressClick.current = true;
    }
    setDragPosition(undefined);
    setDragging(false);
    dragSession.current = undefined;
  };

  const handlePetClick = () => {
    if (suppressClick.current) {
      suppressClick.current = false;
      return;
    }
    toggleOpen();
  };

  return (
    <aside
      className={`${styles.companion} ${embedded ? styles.embedded : styles.roaming} ${atRight ? styles.atRight : ''}`}
      style={embedded ? undefined : motionStyle}
      data-motion={embedded ? 'embedded' : step.id}
      data-animation={animationName}
      data-placement={placedPosition ? 'resting' : dragging ? 'dragging' : 'roaming'}
      aria-label="Kavana, the Caro project companion"
    >
      {open && (
        <section className={styles.dialogue} aria-live="polite">
          <button className={styles.close} type="button" onClick={closeCompanion} aria-label="Close Kavana's message">×</button>
          <span className={styles.eyebrow}>{content.eyebrow}</span>
          <h2>{content.title}</h2>
          <p>{content.body}</p>
          <nav className={styles.topicNav} aria-label="Ask Kavana">
            {(['welcome', 'status', 'adopt', 'hatch'] as Topic[]).map((item) => (
              <button type="button" className={topic === item ? styles.activeTopic : ''} aria-pressed={topic === item} onClick={() => setTopic(item)} key={item}>
                {item === 'welcome' ? 'About' : item === 'adopt' ? 'Adopt me' : item === 'hatch' ? 'Hatch a pet' : 'Status'}
              </button>
            ))}
          </nav>
          <div className={styles.actions}>
            {topic === 'status' && <a href="/roadmap">See the public roadmap <span aria-hidden="true">→</span></a>}
            {topic === 'adopt' && <><a href="https://openai.com/codex/" target="_blank" rel="noreferrer">Get the official Codex app <span aria-hidden="true">↗</span></a><a href="/pets/kavana/kavana-codex-pet.zip" download>Download Kavana <span aria-hidden="true">↓</span></a><a href="/docs/kavana">Installation guide <span aria-hidden="true">→</span></a></>}
            {topic === 'hatch' && <><button type="button" onClick={copyHatchPrompt}>{copied ? 'Prompt copied!' : copyFailed ? 'Copy blocked — open guide' : 'Copy starter prompt'}</button><a href="/docs/kavana#hatch-your-own">Read the pet-making guide <span aria-hidden="true">→</span></a></>}
            {topic === 'welcome' && <button type="button" onClick={() => setTopic('adopt')}>Can I take you home?</button>}
          </div>
          <p className={styles.disclaimer}>Community artwork for Caro · Codex is an OpenAI product</p>
        </section>
      )}
      <button
        type="button"
        className={styles.petButton}
        aria-label={open ? 'Kavana is listening' : 'Talk with Kavana'}
        aria-description={embedded ? undefined : 'Drag Kavana to move her. Release to let her sit, or click to chat.'}
        aria-expanded={open}
        title={embedded ? undefined : 'Drag Kavana to a new spot · click to chat'}
        onClick={handlePetClick}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        onPointerDown={beginDrag}
        onPointerMove={moveDrag}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
      >
        {!open && <span className={styles.prompt}>{placedPosition ? 'Comfy here — drag me anytime' : roamingHints[hintIndex]}</span>}
        <span className={styles.sprite} aria-hidden="true"><img src="/pets/kavana/spritesheet.webp" alt="" draggable={false} style={{ transform: `translate(${-frame.column * 96}px, ${-frame.row * 104}px)` }} /></span>
        <span className={styles.nameplate}>Kavana</span>
      </button>
    </aside>
  );
}
