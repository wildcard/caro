import { db, Waitlist, WaitlistStats } from 'astro:db';

export default async function seed() {
  // Initialize with starting stats
  // The 247 represents early community interest before formal signup
  const today = new Date().toISOString().split('T')[0];

  await db.insert(WaitlistStats).values({
    id: 1,
    date: today,
    signupCount: 0,
    totalCount: 247, // Starting baseline from early community
  });

  // Example seed entries for development (not for production)
  if (import.meta.env.DEV) {
    await db.insert(Waitlist).values([
      {
        id: 1,
        email: 'test@example.com',
        createdAt: new Date(),
        source: 'seed',
        interests: ['guilds', 'runbooks'],
      },
    ]);
  }
}
