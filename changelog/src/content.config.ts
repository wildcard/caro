import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const releases = defineCollection({
  loader: glob({ pattern: '**/*.md', base: './src/content/releases' }),
  schema: ({ image }) =>
    z.object({
      title: z.string(),
      description: z.string(),
      versionNumber: z.string(),
      date: z.coerce.date(),
      breaking: z.boolean().optional().default(false),
      security: z.boolean().optional().default(false),
    }),
});

export const collections = { releases };
