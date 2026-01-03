import { CARO_VERSION } from './version';

export const SITE_CONFIG = {
  name: 'Caro',
  tagline: 'Your loyal shell companion',
  version: CARO_VERSION,
  domain: 'caro.sh',
  github: {
    org: 'wildcard',
    repo: 'caro',
    url: 'https://github.com/wildcard/caro',
  },
  social: {
    x: 'https://x.com/CaroDaShellShib',
    bluesky: 'https://bsky.app/profile/caro-sh.bsky.social',
  },
  downloads: {
    baseUrl: 'https://github.com/wildcard/caro/releases/download',
  },
} as const;
