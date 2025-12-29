/**
 * Social Media Poster Agent
 *
 * Handles automated social media posting for generated content.
 * Supports Twitter/X and can be extended for other platforms.
 */

/**
 * Post content to social media
 */
export async function postToSocial({ content, platforms, config }) {
  const results = {
    success: [],
    errors: [],
  };

  for (const platform of platforms) {
    try {
      const result = await postToPlatform(content, platform, config);
      results.success.push({ platform, ...result });
    } catch (error) {
      results.errors.push({ platform, error: error.message });
    }
  }

  return results;
}

/**
 * Post to specific platform
 */
async function postToPlatform(content, platform, config) {
  switch (platform) {
    case 'twitter':
      return await postToTwitter(content, config);
    case 'mastodon':
      return await postToMastodon(content, config);
    case 'linkedin':
      return await postToLinkedIn(content, config);
    default:
      throw new Error(`Unsupported platform: ${platform}`);
  }
}

/**
 * Post to Twitter/X
 */
async function postToTwitter(content, config) {
  const { socialText, hashtags, url } = content;

  // Build tweet text
  let tweetText = socialText || content.title;

  // Add hashtags if not already included
  if (hashtags && !tweetText.includes('#')) {
    const hashtagText = hashtags.map((h) => `#${h}`).join(' ');
    if (tweetText.length + hashtagText.length + 1 <= 280) {
      tweetText = `${tweetText}\n\n${hashtagText}`;
    }
  }

  // Add URL if provided and space permits
  if (url && tweetText.length + url.length + 1 <= 280) {
    tweetText = `${tweetText}\n${url}`;
  }

  // Truncate if necessary
  if (tweetText.length > 280) {
    tweetText = tweetText.slice(0, 277) + '...';
  }

  // Check for Twitter API credentials
  const apiKey = process.env.TWITTER_API_KEY;
  const apiSecret = process.env.TWITTER_API_SECRET;
  const accessToken = process.env.TWITTER_ACCESS_TOKEN;
  const accessSecret = process.env.TWITTER_ACCESS_SECRET;

  if (!apiKey || !apiSecret || !accessToken || !accessSecret) {
    console.log('Twitter credentials not configured. Tweet preview:');
    console.log('---');
    console.log(tweetText);
    console.log('---');
    return { preview: true, text: tweetText, length: tweetText.length };
  }

  // Post using Twitter API v2
  const response = await fetch('https://api.twitter.com/2/tweets', {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ text: tweetText }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Twitter API error: ${error}`);
  }

  const data = await response.json();
  return {
    posted: true,
    tweetId: data.data?.id,
    text: tweetText,
    url: `https://twitter.com/i/status/${data.data?.id}`,
  };
}

/**
 * Post to Mastodon
 */
async function postToMastodon(content, config) {
  const { socialText, hashtags, url } = content;

  // Build toot text (500 char limit)
  let tootText = socialText || content.title;

  if (hashtags && !tootText.includes('#')) {
    const hashtagText = hashtags.map((h) => `#${h}`).join(' ');
    tootText = `${tootText}\n\n${hashtagText}`;
  }

  if (url) {
    tootText = `${tootText}\n\n${url}`;
  }

  // Truncate if necessary
  if (tootText.length > 500) {
    tootText = tootText.slice(0, 497) + '...';
  }

  const instanceUrl = process.env.MASTODON_INSTANCE_URL;
  const accessToken = process.env.MASTODON_ACCESS_TOKEN;

  if (!instanceUrl || !accessToken) {
    console.log('Mastodon credentials not configured. Toot preview:');
    console.log('---');
    console.log(tootText);
    console.log('---');
    return { preview: true, text: tootText, length: tootText.length };
  }

  const response = await fetch(`${instanceUrl}/api/v1/statuses`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ status: tootText }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Mastodon API error: ${error}`);
  }

  const data = await response.json();
  return {
    posted: true,
    statusId: data.id,
    text: tootText,
    url: data.url,
  };
}

/**
 * Post to LinkedIn
 */
async function postToLinkedIn(content, config) {
  const { socialText, url } = content;

  // Build LinkedIn post
  let postText = socialText || content.title;

  if (url) {
    postText = `${postText}\n\n${url}`;
  }

  const accessToken = process.env.LINKEDIN_ACCESS_TOKEN;

  if (!accessToken) {
    console.log('LinkedIn credentials not configured. Post preview:');
    console.log('---');
    console.log(postText);
    console.log('---');
    return { preview: true, text: postText, length: postText.length };
  }

  // Note: LinkedIn API requires OAuth 2.0 and user/organization ID
  // This is a placeholder for the actual implementation
  throw new Error('LinkedIn posting not yet implemented');
}

/**
 * Generate social media text from content
 */
export function generateSocialText(content, platform) {
  const maxLength = {
    twitter: 280,
    mastodon: 500,
    linkedin: 3000,
  };

  const limit = maxLength[platform] || 280;

  // Extract key info
  const title = content.title || content.brief?.title;
  const description = content.description || content.brief?.description;
  const command = content.command || content.brief?.command;
  const hashtags = content.hashtags || content.brief?.hashtags || ['unix', 'cli', 'caro'];

  // Build text based on content type
  let text = '';

  if (command) {
    text = `Today's Unix command: ${command}\n\n${description || title}`;
  } else {
    text = title;
    if (description && text.length + description.length < limit - 50) {
      text = `${text}\n\n${description}`;
    }
  }

  // Add hashtags
  const hashtagText = hashtags.slice(0, 5).map((h) => `#${h}`).join(' ');
  if (text.length + hashtagText.length + 2 < limit) {
    text = `${text}\n\n${hashtagText}`;
  }

  // Truncate if needed
  if (text.length > limit) {
    text = text.slice(0, limit - 3) + '...';
  }

  return text;
}

/**
 * Schedule post for later
 */
export async function schedulePost({ content, platform, scheduledTime, config }) {
  // This would integrate with a scheduling service
  // For now, return scheduled info for manual posting
  return {
    scheduled: true,
    platform,
    scheduledTime,
    content: generateSocialText(content, platform),
  };
}

/**
 * Get optimal posting times
 */
export function getOptimalPostingTime(platform, timezone = 'America/Los_Angeles') {
  // Best times based on general social media research
  const optimalTimes = {
    twitter: { hour: 9, minute: 0 }, // 9 AM
    mastodon: { hour: 10, minute: 0 }, // 10 AM
    linkedin: { hour: 8, minute: 30 }, // 8:30 AM
  };

  const time = optimalTimes[platform] || { hour: 9, minute: 0 };

  const now = new Date();
  const scheduled = new Date(now);
  scheduled.setHours(time.hour, time.minute, 0, 0);

  // If time has passed today, schedule for tomorrow
  if (scheduled < now) {
    scheduled.setDate(scheduled.getDate() + 1);
  }

  return scheduled;
}
