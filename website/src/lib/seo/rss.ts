/**
 * /rss.xml for the blog, built from the posts in the build output: every
 * `/blog/<post>` page with an `article:published_time`, newest first. A new
 * post needs no registration here.
 */
import { cardTitle } from './routes';

export interface RssPost {
  url: string;
  title: string;
  description: string;
  publishedTime: string;
  image?: string;
  /** The image's size in bytes; RSS requires it on an enclosure. */
  imageLength?: number;
  /** The image's MIME type, e.g. `image/png`. */
  imageType?: string;
}

function xml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export function buildRss(posts: RssPost[], site: URL): string {
  const items = [...posts]
    .filter((post) => !Number.isNaN(new Date(post.publishedTime).getTime()))
    .sort((a, b) => Date.parse(b.publishedTime) - Date.parse(a.publishedTime))
    .map((post) =>
      [
        '    <item>',
        `      <title>${xml(cardTitle(post.title))}</title>`,
        `      <link>${xml(post.url)}</link>`,
        `      <guid isPermaLink="true">${xml(post.url)}</guid>`,
        `      <description>${xml(post.description)}</description>`,
        `      <pubDate>${new Date(post.publishedTime).toUTCString()}</pubDate>`,
        post.image && post.imageLength && post.imageType
          ? `      <enclosure url="${xml(post.image)}" type="${post.imageType}" length="${post.imageLength}"/>`
          : '',
        '    </item>',
      ]
        .filter(Boolean)
        .join('\n'),
    );
  const blog = new URL('/blog', site).href;
  return `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>Caro Blog</title>
    <link>${xml(blog)}</link>
    <description>Stories, releases and engineering notes from Caro, your loyal shell companion.</description>
    <language>en</language>
    <atom:link href="${xml(new URL('/rss.xml', site).href)}" rel="self" type="application/rss+xml"/>
${items.join('\n')}
  </channel>
</rss>
`;
}
