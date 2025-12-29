import { config, fields, collection, singleton } from '@keystatic/core';

/**
 * Keystatic CMS Configuration
 *
 * This separates content management from templates, allowing:
 * - Visual editing through /keystatic admin UI
 * - Git-based storage (all content stored as files)
 * - Type-safe content schemas
 * - Easy content updates without code changes
 */

export default config({
  storage: {
    // Local mode for development, GitHub mode for production
    kind: 'local',
  },

  ui: {
    brand: {
      name: 'Caro Learn CMS',
    },
    navigation: {
      'Content': ['commands', 'stories', 'dailyPicks'],
      'Settings': ['siteSettings'],
    },
  },

  singletons: {
    /**
     * Site-wide settings
     */
    siteSettings: singleton({
      label: 'Site Settings',
      path: 'src/data/settings/site',
      schema: {
        siteName: fields.text({
          label: 'Site Name',
          validation: { isRequired: true },
        }),
        siteDescription: fields.text({
          label: 'Site Description',
          multiline: true,
        }),
        socialLinks: fields.object({
          twitter: fields.text({ label: 'Twitter URL' }),
          mastodon: fields.text({ label: 'Mastodon URL' }),
          github: fields.text({ label: 'GitHub URL' }),
        }),
        newsletter: fields.object({
          enabled: fields.checkbox({ label: 'Newsletter Enabled' }),
          provider: fields.select({
            label: 'Newsletter Provider',
            options: [
              { label: 'Buttondown', value: 'buttondown' },
              { label: 'Substack', value: 'substack' },
              { label: 'ConvertKit', value: 'convertkit' },
            ],
            defaultValue: 'buttondown',
          }),
          signupUrl: fields.text({ label: 'Signup URL' }),
        }),
      },
    }),
  },

  collections: {
    /**
     * Command Tutorials Collection
     * Terminus-style practical command tutorials
     */
    commands: collection({
      label: 'Command Tutorials',
      slugField: 'command',
      path: 'src/content/commands/*',
      format: { contentField: 'content' },
      schema: {
        // Metadata
        title: fields.text({
          label: 'Title',
          description: 'Full title including command name (e.g., "xargs: Transform Input into Arguments")',
          validation: { isRequired: true },
        }),
        command: fields.slug({
          name: {
            label: 'Command',
            description: 'The command name (e.g., xargs, find, grep)',
            validation: { isRequired: true },
          },
        }),
        description: fields.text({
          label: 'Description',
          description: 'Brief description for SEO and previews',
          multiline: true,
          validation: { isRequired: true },
        }),
        difficulty: fields.select({
          label: 'Difficulty Level',
          options: [
            { label: 'Beginner', value: 'beginner' },
            { label: 'Intermediate', value: 'intermediate' },
            { label: 'Advanced', value: 'advanced' },
          ],
          defaultValue: 'beginner',
        }),
        platforms: fields.multiselect({
          label: 'Supported Platforms',
          options: [
            { label: 'Linux', value: 'linux' },
            { label: 'macOS', value: 'macos' },
            { label: 'BSD', value: 'bsd' },
            { label: 'POSIX', value: 'posix' },
            { label: 'Unix', value: 'unix' },
          ],
        }),
        tags: fields.array(
          fields.text({ label: 'Tag' }),
          {
            label: 'Tags',
            itemLabel: (props) => props.value || 'New Tag',
          }
        ),
        publishedAt: fields.date({
          label: 'Published Date',
          validation: { isRequired: true },
        }),
        featured: fields.checkbox({
          label: 'Featured',
          description: 'Show on homepage and Learn hub',
          defaultValue: false,
        }),

        // Relationships
        relatedCommands: fields.array(
          fields.text({ label: 'Command' }),
          {
            label: 'Related Commands',
            description: 'Other commands to link to',
            itemLabel: (props) => props.value || 'New Command',
          }
        ),
        caroPrompt: fields.text({
          label: 'Caro Example Prompt',
          description: 'Example prompt users can try with Caro',
        }),

        // Content
        content: fields.mdx({
          label: 'Content',
          description: 'Tutorial content in MDX format',
        }),
      },
    }),

    /**
     * Unix History Stories Collection
     */
    stories: collection({
      label: 'Unix Stories',
      slugField: 'slug',
      path: 'src/content/stories/*',
      format: { contentField: 'content' },
      schema: {
        slug: fields.slug({
          name: {
            label: 'Slug',
            validation: { isRequired: true },
          },
        }),
        title: fields.text({
          label: 'Title',
          validation: { isRequired: true },
        }),
        subtitle: fields.text({
          label: 'Subtitle',
          description: 'Optional subtitle or tagline',
        }),
        category: fields.select({
          label: 'Category',
          options: [
            { label: 'Unix History', value: 'history' },
            { label: 'People & Pioneers', value: 'people' },
            { label: 'Technology', value: 'technology' },
            { label: 'Unix Culture', value: 'culture' },
            { label: 'Platform Spotlight', value: 'platform' },
          ],
          defaultValue: 'history',
        }),
        era: fields.text({
          label: 'Era',
          description: 'Time period (e.g., "1970s", "Modern", "1990s-2000s")',
          validation: { isRequired: true },
        }),
        publishedAt: fields.date({
          label: 'Published Date',
          validation: { isRequired: true },
        }),
        featured: fields.checkbox({
          label: 'Featured',
          defaultValue: false,
        }),
        readingTime: fields.integer({
          label: 'Reading Time (minutes)',
          validation: { isRequired: true, min: 1 },
        }),
        author: fields.text({
          label: 'Author',
          defaultValue: 'Caro Team',
        }),
        sources: fields.array(
          fields.object({
            title: fields.text({ label: 'Source Title' }),
            url: fields.url({ label: 'Source URL' }),
          }),
          {
            label: 'Sources & References',
            itemLabel: (props) => props.fields.title.value || 'New Source',
          }
        ),
        tags: fields.array(
          fields.text({ label: 'Tag' }),
          {
            label: 'Tags',
            itemLabel: (props) => props.value || 'New Tag',
          }
        ),

        // Content
        content: fields.mdx({
          label: 'Story Content',
        }),
      },
    }),

    /**
     * Daily Picks Collection
     * Short-form content for social media
     */
    dailyPicks: collection({
      label: 'Daily Picks',
      slugField: 'slug',
      path: 'src/content/daily-picks/*',
      format: { contentField: 'content' },
      schema: {
        slug: fields.slug({
          name: {
            label: 'Slug',
            description: 'URL-friendly identifier (e.g., 2025-01-15-lsof)',
            validation: { isRequired: true },
          },
        }),
        title: fields.text({
          label: 'Title',
          validation: { isRequired: true },
        }),
        type: fields.select({
          label: 'Type',
          options: [
            { label: 'Command of the Day', value: 'command' },
            { label: 'Pro Tip', value: 'tip' },
            { label: 'Did You Know?', value: 'trivia' },
            { label: 'Unix Wisdom', value: 'quote' },
            { label: 'Error Archaeology', value: 'error' },
          ],
          defaultValue: 'command',
        }),
        publishedAt: fields.date({
          label: 'Published Date',
          validation: { isRequired: true },
        }),
        socialText: fields.text({
          label: 'Social Media Text',
          description: 'Pre-formatted text for Twitter/social sharing (max 280 chars)',
          multiline: true,
          validation: { isRequired: true },
        }),
        hashtags: fields.array(
          fields.text({ label: 'Hashtag (without #)' }),
          {
            label: 'Hashtags',
            itemLabel: (props) => props.value ? `#${props.value}` : 'New Hashtag',
          }
        ),
        source: fields.text({
          label: 'Source',
          description: 'Attribution or source reference',
        }),

        // Content
        content: fields.mdx({
          label: 'Full Content',
          description: 'Expanded content for the website',
        }),
      },
    }),
  },
});
