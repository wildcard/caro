import { defineDb, defineTable, column } from 'astro:db';

/**
 * Waitlist table for the Caro community platform
 * Stores email signups for early access to the DevRel Platform / Web Hub
 */
const Waitlist = defineTable({
  columns: {
    id: column.number({ primaryKey: true }),
    email: column.text({ unique: true }),
    createdAt: column.date({ default: new Date() }),
    source: column.text({ optional: true }), // Where they signed up from (landing, blog, etc.)
    referrer: column.text({ optional: true }), // HTTP referrer
    interests: column.json({ optional: true }), // Array of interests: ["guilds", "runbooks", "sharing"]
  },
});

/**
 * Analytics for waitlist metrics
 * Tracks daily signup counts for dashboard
 */
const WaitlistStats = defineTable({
  columns: {
    id: column.number({ primaryKey: true }),
    date: column.text(), // YYYY-MM-DD format
    signupCount: column.number({ default: 0 }),
    totalCount: column.number({ default: 0 }),
  },
});

export default defineDb({
  tables: { Waitlist, WaitlistStats },
});
