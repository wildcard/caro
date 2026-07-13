import { useEffect, useState } from 'react';
import styles from './KavanaCompanion.module.css';

type Topic = 'welcome' | 'status' | 'adopt' | 'hatch';

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

const roamingHints = ['Psst—need a guide?', 'Roadmap check-in ready', 'Want a Codex pet?'];

export function KavanaCompanion({ embedded = false, initiallyOpen = false }: KavanaCompanionProps) {
  const [open, setOpen] = useState(initiallyOpen);
  const [topic, setTopic] = useState<Topic>('welcome');
  const [frame, setFrame] = useState(0);
  const [atRight, setAtRight] = useState(false);
  const [copied, setCopied] = useState(false);
  const [hintIndex, setHintIndex] = useState(0);

  useEffect(() => {
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    const animation = window.setInterval(() => setFrame((value) => (value + 1) % 8), 120);
    return () => window.clearInterval(animation);
  }, []);

  useEffect(() => {
    if (embedded || open || window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    const roam = window.setInterval(() => setAtRight((value) => !value), 9000);
    return () => window.clearInterval(roam);
  }, [embedded, open]);

  useEffect(() => {
    if (embedded || open) return;
    const hints = window.setInterval(() => setHintIndex((value) => (value + 1) % roamingHints.length), 6000);
    return () => window.clearInterval(hints);
  }, [embedded, open]);

  const copyHatchPrompt = async () => {
    await navigator.clipboard.writeText('Use the hatch-pet skill to create a Codex pet from my idea or attached reference art. Package and validate the full v2 animated spritesheet, and show me the final QA preview.');
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1800);
  };

  const animationRow = open ? 0 : atRight ? 1 : 2;
  const content = topics[topic];

  return (
    <aside className={`${styles.companion} ${embedded ? styles.embedded : styles.roaming} ${atRight ? styles.atRight : ''}`} aria-label="Kavana, the Caro project companion">
      {open && (
        <section className={styles.dialogue} aria-live="polite">
          <button className={styles.close} type="button" onClick={() => setOpen(false)} aria-label="Close Kavana's message">×</button>
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
            {topic === 'hatch' && <><button type="button" onClick={copyHatchPrompt}>{copied ? 'Prompt copied!' : 'Copy starter prompt'}</button><a href="/docs/kavana#hatch-your-own">Read the pet-making guide <span aria-hidden="true">→</span></a></>}
            {topic === 'welcome' && <button type="button" onClick={() => setTopic('adopt')}>Can I take you home?</button>}
          </div>
          <p className={styles.disclaimer}>Community artwork for Caro · Codex is an OpenAI product</p>
        </section>
      )}
      <button type="button" className={styles.petButton} aria-label={open ? 'Kavana is listening' : 'Talk with Kavana'} aria-expanded={open} onClick={() => setOpen((value) => !value)}>
        {!open && <span className={styles.prompt}>{roamingHints[hintIndex]}</span>}
        <span className={styles.sprite} aria-hidden="true"><img src="/pets/kavana/spritesheet.webp" alt="" draggable={false} style={{ transform: `translate(${-frame * 96}px, ${-animationRow * 104}px)` }} /></span>
        <span className={styles.nameplate}>Kavana</span>
      </button>
    </aside>
  );
}
