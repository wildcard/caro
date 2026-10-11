/**
 * Static SVG art for occasion themes: /occasions/art/<id>-<variant>.svg
 * Prerendered at build time (output: 'static'), one file per occasion and
 * variant. See src/lib/occasion-art.ts.
 */
import type { APIRoute, GetStaticPaths } from 'astro';
import { OCCASIONS, getOccasion } from '../../../config/occasions';
import { ART_VARIANTS, renderOccasionArt, type ArtVariant } from '../../../lib/occasion-art';

export const getStaticPaths: GetStaticPaths = () =>
  OCCASIONS.flatMap((o) => ART_VARIANTS.map((v) => ({ params: { name: `${o.id}-${v}` } })));

export const GET: APIRoute = ({ params }) => {
  const name = params.name ?? '';
  const variant = ART_VARIANTS.find((v) => name.endsWith(`-${v}`)) as ArtVariant;
  const occasion = getOccasion(name.slice(0, -(variant.length + 1)));
  if (!occasion) return new Response('Not found', { status: 404 });
  return new Response(renderOccasionArt(occasion, variant), {
    headers: { 'Content-Type': 'image/svg+xml; charset=utf-8' },
  });
};
