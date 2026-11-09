import "server-only";
import type { FeedCard, Friend, Profile } from "./types";

const API_BASE = "https://www.duolingo.com/2017-06-30";
const FRIENDS_API_BASE = "https://www.duolingo.com/2023-05-23";

/**
 * The reaction value the real app's "Celebrate" (FEIERN) button sends. Verified live:
 * posting this to a zero-reaction card made it show up as `reactionCounts: {"congrats": 1}`.
 */
export const CELEBRATE_REACTION = "CONGRATS";

function jwt(): string {
  const token = process.env.DUOLINGO_JWT_TOKEN;
  if (!token) {
    throw new Error("DUOLINGO_JWT_TOKEN is not set (see .env.local.example)");
  }
  return token;
}

/** Decodes the JWT's `sub` claim -- same manual approach as rust/core/src/api.rs, no JWT library needed. */
function userIdFromJwt(token: string): number {
  const payload = token.split(".")[1];
  if (!payload) throw new Error("JWT is missing its payload segment");

  const json = Buffer.from(payload, "base64url").toString("utf-8");
  const claims = JSON.parse(json) as { sub?: number };
  if (typeof claims.sub !== "number") {
    throw new Error("JWT payload has no numeric 'sub' claim");
  }
  return claims.sub;
}

function userId(): number {
  return userIdFromJwt(jwt());
}

async function get<T>(url: string): Promise<T> {
  console.log(`[duolingo] GET ${url}`);
  const response = await fetch(url, {
    headers: { Authorization: `Bearer ${jwt()}` },
    cache: "no-store",
  });

  if (!response.ok) {
    const msg = `Duolingo API returned HTTP ${response.status} for ${url}`;
    console.warn(`[duolingo] ${msg}`);
    throw new Error(msg);
  }

  return (await response.json()) as T;
}

export async function getProfile(): Promise<Profile> {
  return get<Profile>(`${API_BASE}/users/${userId()}`);
}

/** People the logged-in user follows. */
export async function getFollowing(): Promise<Friend[]> {
  const resp = await get<{ subscriptions: Friend[] }>(`${API_BASE}/users/${userId()}/subscriptions`);
  console.log(`[duolingo] fetched ${resp.subscriptions.length} following`);
  return resp.subscriptions;
}

/** People who follow the logged-in user. */
export async function getFollowers(): Promise<Friend[]> {
  const resp = await get<{ subscribers: Friend[] }>(`${API_BASE}/users/${userId()}/subscribers`);
  console.log(`[duolingo] fetched ${resp.subscribers.length} followers`);
  return resp.subscribers;
}

/** The friends activity feed: league promotions, kudos, follows, etc., most recent first. */
export async function getFeed(): Promise<FeedCard[]> {
  const resp = await get<{ feed: { feedCards: FeedCard[] }[] }>(
    `${FRIENDS_API_BASE}/friends/users/${userId()}/feed/v2?uiLanguage=en`
  );
  const cards = resp.feed.flatMap((group) => group.feedCards);
  cards.sort((a, b) => b.timestamp - a.timestamp);
  console.log(`[duolingo] fetched ${cards.length} feed cards`);
  return cards;
}

/**
 * React to a feed card (e.g. the real app's "Celebrate" button sends `"CONGRATS"`).
 * Verified live against the real API: a zero-reaction card went from no `congrats` entry
 * to `{"congrats": 1}` after calling this with `reaction = "CONGRATS"`.
 */
export async function react(eventId: string, reaction: string): Promise<void> {
  const url = "https://www.duolingo.com/card/reaction";
  console.log(`[duolingo] POST ${url} (reaction=${reaction}, event_id=${eventId})`);

  const response = await fetch(url, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${jwt()}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      groupId: eventId,
      reaction,
      trackingProperties: { screen: "kudos_feed" },
      userId: userId(),
    }),
  });

  if (!response.ok) {
    const msg = `Duolingo API returned HTTP ${response.status} for ${url}`;
    console.warn(`[duolingo] ${msg}`);
    throw new Error(msg);
  }
}
