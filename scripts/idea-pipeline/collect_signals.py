#!/usr/bin/env python3
"""
Signal Collection for Agentic Idea Pipeline

Collects signals from various sources:
- Hacker News (via Algolia API)
- Reddit (via PRAW)
- RSS feeds

Usage:
    python collect_signals.py --output signals.json --sources hackernews,reddit,rss
"""

import argparse
import json
import os
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Optional

import feedparser
import httpx
import yaml
from pydantic import BaseModel

# Optional: Reddit support
try:
    import praw
    REDDIT_AVAILABLE = True
except ImportError:
    REDDIT_AVAILABLE = False


class Signal(BaseModel):
    """A single signal from any source."""
    id: str
    source: str  # hackernews, reddit, rss, twitter
    title: str
    url: Optional[str] = None
    content: Optional[str] = None
    author: Optional[str] = None
    score: int = 0
    comments: int = 0
    timestamp: datetime
    query_matched: Optional[str] = None
    subreddit: Optional[str] = None
    tags: list[str] = []


class SignalCollection(BaseModel):
    """Collection of signals from a pipeline run."""
    collected_at: datetime
    lookback_hours: int
    sources_queried: list[str]
    signals: list[Signal]
    stats: dict


def load_queries(queries_file: Path = None) -> dict:
    """Load query configuration from YAML file."""
    if queries_file is None:
        queries_file = Path(__file__).parent / "queries.yaml"

    with open(queries_file) as f:
        return yaml.safe_load(f)


def collect_hackernews(queries: dict, lookback_hours: int) -> list[Signal]:
    """Collect signals from Hacker News via Algolia API."""
    signals = []
    config = queries.get("hackernews", {})
    search_endpoint = config.get("search_endpoint", "https://hn.algolia.com/api/v1/search")
    min_points = config.get("min_points", 10)

    # Calculate timestamp for lookback
    cutoff = datetime.now(timezone.utc) - timedelta(hours=lookback_hours)
    cutoff_ts = int(cutoff.timestamp())

    # Collect queries from all categories
    all_queries = []
    for category, data in queries.items():
        if isinstance(data, dict) and "queries" in data:
            if "hackernews" in data.get("sources", []):
                all_queries.extend(data["queries"])

    # Deduplicate queries
    all_queries = list(set(all_queries))

    print(f"[HN] Searching {len(all_queries)} queries...")

    for query in all_queries:
        try:
            response = httpx.get(
                search_endpoint,
                params={
                    "query": query,
                    "tags": "(story,show_hn,ask_hn)",
                    "numericFilters": f"created_at_i>{cutoff_ts},points>{min_points}",
                    "hitsPerPage": 50,
                },
                timeout=30,
            )
            response.raise_for_status()
            data = response.json()

            for hit in data.get("hits", []):
                signal = Signal(
                    id=f"hn-{hit['objectID']}",
                    source="hackernews",
                    title=hit.get("title", ""),
                    url=hit.get("url") or f"https://news.ycombinator.com/item?id={hit['objectID']}",
                    content=hit.get("story_text"),
                    author=hit.get("author"),
                    score=hit.get("points", 0),
                    comments=hit.get("num_comments", 0),
                    timestamp=datetime.fromtimestamp(hit["created_at_i"], tz=timezone.utc),
                    query_matched=query,
                    tags=hit.get("_tags", []),
                )
                signals.append(signal)

        except Exception as e:
            print(f"[HN] Error searching '{query}': {e}", file=sys.stderr)

    # Deduplicate by ID
    seen = set()
    unique_signals = []
    for s in signals:
        if s.id not in seen:
            seen.add(s.id)
            unique_signals.append(s)

    print(f"[HN] Collected {len(unique_signals)} unique signals")
    return unique_signals


def collect_reddit(queries: dict, lookback_hours: int) -> list[Signal]:
    """Collect signals from Reddit via PRAW."""
    if not REDDIT_AVAILABLE:
        print("[Reddit] PRAW not installed, skipping Reddit collection", file=sys.stderr)
        return []

    client_id = os.environ.get("REDDIT_CLIENT_ID")
    client_secret = os.environ.get("REDDIT_CLIENT_SECRET")

    if not client_id or not client_secret:
        print("[Reddit] Missing API credentials, skipping", file=sys.stderr)
        return []

    signals = []

    try:
        reddit = praw.Reddit(
            client_id=client_id,
            client_secret=client_secret,
            user_agent="caro-idea-pipeline/1.0",
        )

        subreddits = queries.get("reddit_subreddits", [])
        cutoff = datetime.now(timezone.utc) - timedelta(hours=lookback_hours)

        for sub_config in subreddits:
            sub_name = sub_config["name"]
            keywords = sub_config.get("keywords", [])

            print(f"[Reddit] Scanning r/{sub_name}...")

            try:
                subreddit = reddit.subreddit(sub_name)
                for post in subreddit.hot(limit=100):
                    # Check if post is within lookback window
                    post_time = datetime.fromtimestamp(post.created_utc, tz=timezone.utc)
                    if post_time < cutoff:
                        continue

                    # Check if post matches any keywords
                    title_lower = post.title.lower()
                    selftext_lower = (post.selftext or "").lower()

                    matches_keyword = not keywords  # If no keywords, match all
                    for kw in keywords:
                        if kw.lower() in title_lower or kw.lower() in selftext_lower:
                            matches_keyword = True
                            break

                    if not matches_keyword:
                        continue

                    signal = Signal(
                        id=f"reddit-{post.id}",
                        source="reddit",
                        title=post.title,
                        url=f"https://reddit.com{post.permalink}",
                        content=post.selftext[:1000] if post.selftext else None,
                        author=str(post.author) if post.author else None,
                        score=post.score,
                        comments=post.num_comments,
                        timestamp=post_time,
                        subreddit=sub_name,
                        tags=[f"r/{sub_name}"],
                    )
                    signals.append(signal)

            except Exception as e:
                print(f"[Reddit] Error scanning r/{sub_name}: {e}", file=sys.stderr)

    except Exception as e:
        print(f"[Reddit] Connection error: {e}", file=sys.stderr)
        return []

    # Deduplicate
    seen = set()
    unique_signals = []
    for s in signals:
        if s.id not in seen:
            seen.add(s.id)
            unique_signals.append(s)

    print(f"[Reddit] Collected {len(unique_signals)} unique signals")
    return unique_signals


def collect_rss(queries: dict, lookback_hours: int) -> list[Signal]:
    """Collect signals from RSS feeds."""
    signals = []
    feeds = queries.get("rss_feeds", [])
    cutoff = datetime.now(timezone.utc) - timedelta(hours=lookback_hours)

    for feed_config in feeds:
        feed_name = feed_config["name"]
        feed_url = feed_config["url"]
        keywords = feed_config.get("filter_keywords", [])

        print(f"[RSS] Fetching {feed_name}...")

        try:
            feed = feedparser.parse(feed_url)

            for entry in feed.entries:
                # Parse date
                entry_time = None
                if hasattr(entry, "published_parsed") and entry.published_parsed:
                    entry_time = datetime(*entry.published_parsed[:6], tzinfo=timezone.utc)
                elif hasattr(entry, "updated_parsed") and entry.updated_parsed:
                    entry_time = datetime(*entry.updated_parsed[:6], tzinfo=timezone.utc)
                else:
                    entry_time = datetime.now(timezone.utc)

                # Check if within lookback window
                if entry_time < cutoff:
                    continue

                # Check keyword filter
                title_lower = entry.get("title", "").lower()
                summary_lower = entry.get("summary", "").lower()

                matches_keyword = not keywords  # If no keywords, match all
                for kw in keywords:
                    if kw.lower() in title_lower or kw.lower() in summary_lower:
                        matches_keyword = True
                        break

                if not matches_keyword:
                    continue

                signal = Signal(
                    id=f"rss-{hash(entry.get('link', entry.get('title', '')))}",
                    source="rss",
                    title=entry.get("title", "No title"),
                    url=entry.get("link"),
                    content=entry.get("summary", "")[:1000],
                    author=entry.get("author"),
                    score=0,
                    comments=0,
                    timestamp=entry_time,
                    tags=[feed_name],
                )
                signals.append(signal)

        except Exception as e:
            print(f"[RSS] Error fetching {feed_name}: {e}", file=sys.stderr)

    # Deduplicate by URL
    seen = set()
    unique_signals = []
    for s in signals:
        key = s.url or s.title
        if key not in seen:
            seen.add(key)
            unique_signals.append(s)

    print(f"[RSS] Collected {len(unique_signals)} unique signals")
    return unique_signals


def main():
    parser = argparse.ArgumentParser(description="Collect signals for idea pipeline")
    parser.add_argument("--output", "-o", default="signals.json", help="Output file")
    parser.add_argument("--sources", default="hackernews,reddit,rss", help="Comma-separated sources")
    parser.add_argument("--lookback-hours", type=int, default=24, help="Hours to look back")
    parser.add_argument("--queries-file", type=Path, help="Custom queries YAML file")
    args = parser.parse_args()

    sources = [s.strip().lower() for s in args.sources.split(",")]
    queries = load_queries(args.queries_file)

    all_signals = []

    if "hackernews" in sources:
        all_signals.extend(collect_hackernews(queries, args.lookback_hours))

    if "reddit" in sources:
        all_signals.extend(collect_reddit(queries, args.lookback_hours))

    if "rss" in sources:
        all_signals.extend(collect_rss(queries, args.lookback_hours))

    # Sort by score (descending) then timestamp (descending)
    all_signals.sort(key=lambda s: (-s.score, -s.timestamp.timestamp()))

    collection = SignalCollection(
        collected_at=datetime.now(timezone.utc),
        lookback_hours=args.lookback_hours,
        sources_queried=sources,
        signals=all_signals,
        stats={
            "total": len(all_signals),
            "by_source": {
                source: len([s for s in all_signals if s.source == source])
                for source in set(s.source for s in all_signals)
            },
        },
    )

    with open(args.output, "w") as f:
        json.dump(collection.model_dump(mode="json"), f, indent=2, default=str)

    print(f"\nTotal signals collected: {len(all_signals)}")
    print(f"Output written to: {args.output}")


if __name__ == "__main__":
    main()
