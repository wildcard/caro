import type { APIRoute } from 'astro';
import { db, Waitlist, WaitlistStats, eq, count } from 'astro:db';

export const prerender = false;

export const POST: APIRoute = async ({ request }) => {
  try {
    const body = await request.json();
    const { email } = body;

    // Validate email
    if (!email || typeof email !== 'string') {
      return new Response(
        JSON.stringify({ error: 'email_required' }),
        { status: 400, headers: { 'Content-Type': 'application/json' } }
      );
    }

    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email)) {
      return new Response(
        JSON.stringify({ error: 'invalid_email' }),
        { status: 400, headers: { 'Content-Type': 'application/json' } }
      );
    }

    // Check if email already exists
    const existing = await db
      .select()
      .from(Waitlist)
      .where(eq(Waitlist.email, email.toLowerCase()))
      .limit(1);

    if (existing.length > 0) {
      // Already subscribed - return success anyway
      const totalCount = await getTotalCount();
      return new Response(
        JSON.stringify({ success: true, error: 'already_subscribed', count: totalCount }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      );
    }

    // Get referrer from request
    const referrer = request.headers.get('referer') || '';
    const source = getSourceFromReferrer(referrer);

    // Insert new signup
    await db.insert(Waitlist).values({
      email: email.toLowerCase(),
      createdAt: new Date(),
      source,
      referrer,
    });

    // Update stats
    await updateStats();

    // Get updated count
    const totalCount = await getTotalCount();

    return new Response(
      JSON.stringify({ success: true, count: totalCount }),
      { status: 200, headers: { 'Content-Type': 'application/json' } }
    );
  } catch (error) {
    console.error('Waitlist signup error:', error);

    // Check for unique constraint violation
    if (error instanceof Error && error.message.includes('UNIQUE')) {
      const totalCount = await getTotalCount();
      return new Response(
        JSON.stringify({ success: true, error: 'already_subscribed', count: totalCount }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      );
    }

    return new Response(
      JSON.stringify({ error: 'server_error' }),
      { status: 500, headers: { 'Content-Type': 'application/json' } }
    );
  }
};

async function getTotalCount(): Promise<number> {
  try {
    const stats = await db.select().from(WaitlistStats).limit(1);
    if (stats.length > 0) {
      return stats[0].totalCount;
    }

    // Fallback: count actual signups + baseline
    const signupCount = await db.select({ count: count() }).from(Waitlist);
    return 247 + (signupCount[0]?.count || 0);
  } catch {
    return 247;
  }
}

async function updateStats(): Promise<void> {
  try {
    const today = new Date().toISOString().split('T')[0];
    const signupCount = await db.select({ count: count() }).from(Waitlist);
    const totalCount = 247 + (signupCount[0]?.count || 0);

    // Try to update existing stats
    const existing = await db.select().from(WaitlistStats).limit(1);

    if (existing.length > 0) {
      await db
        .update(WaitlistStats)
        .set({
          date: today,
          signupCount: signupCount[0]?.count || 0,
          totalCount,
        })
        .where(eq(WaitlistStats.id, existing[0].id));
    } else {
      await db.insert(WaitlistStats).values({
        date: today,
        signupCount: signupCount[0]?.count || 0,
        totalCount,
      });
    }
  } catch (error) {
    console.error('Error updating stats:', error);
  }
}

function getSourceFromReferrer(referrer: string): string {
  if (!referrer) return 'direct';

  try {
    const url = new URL(referrer);
    const path = url.pathname;

    if (path === '/' || path === '') return 'landing';
    if (path.startsWith('/blog')) return 'blog';
    if (path.startsWith('/explore')) return 'explore';
    if (path.startsWith('/use-cases')) return 'use-cases';
    if (path.startsWith('/compare')) return 'compare';

    return 'other';
  } catch {
    return 'unknown';
  }
}

// Optional: GET endpoint for fetching current count
export const GET: APIRoute = async () => {
  try {
    const totalCount = await getTotalCount();
    return new Response(
      JSON.stringify({ count: totalCount }),
      { status: 200, headers: { 'Content-Type': 'application/json' } }
    );
  } catch (error) {
    console.error('Error fetching waitlist count:', error);
    return new Response(
      JSON.stringify({ count: 247 }),
      { status: 200, headers: { 'Content-Type': 'application/json' } }
    );
  }
};
