import {
  Navigation,
  Hero,
  Features,
  Documentation,
  Contributors,
  Footer,
} from '@/components';

export default function Home() {
  return (
    <div className="min-h-screen bg-pixel-bg-primary">
      <Navigation />
      <main>
        <Hero />
        <Features />
        <Documentation />
        <Contributors />
      </main>
      <Footer />
    </div>
  );
}
