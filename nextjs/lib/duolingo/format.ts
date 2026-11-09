// Pure formatting helpers mirroring rust/core/src/format.rs. No framework types here --
// plain strings/numbers, safe to import from Client or Server Components alike.

import type { FeedCard } from "./types";

/**
 * The card's displayable achievement/update text. Most card types already have this in
 * `body`; SHARE_SENTENCE_OFFER is the one confirmed exception -- `body` is `""` there,
 * with the actual content in `fromSentence`/`toSentence` instead. Found live: a raw card
 * showed the literal string "SHARE_SENTENCE_OFFER" to the user before this existed,
 * because the old fallback (`body || cardType`) had nothing better to fall back to.
 */
export function cardBody(card: FeedCard): string {
  if (card.cardType === "SHARE_SENTENCE_OFFER" && card.fromSentence && card.toSentence) {
    return `"${card.fromSentence}" \u{2192} "${card.toSentence}"`;
  }
  return card.body || card.cardType;
}

/**
 * Whether the Celebrate button should be shown at all. `eventId` alone isn't enough --
 * found live: FOLLOW cards have a real eventId but `reactionCounts: null` (Duolingo
 * tracks no reaction mechanism for them at all), and SHARE_SENTENCE_OFFER cards use
 * `like`/`haha`/`love` reactions, not `congrats` -- sending CONGRATS (the only reaction
 * value actually verified against the live API, see core/api.ts) would be wrong for
 * either. Only show it where CONGRATS is the type of reaction the real app itself would
 * send for a bare click.
 */
export function canCelebrate(card: FeedCard): boolean {
  return Boolean(card.eventId) && (card.cardType === "KUDOS_OFFER" || card.cardType === "KUDOS_MILESTONE");
}

export function reactionEmoji(name: string): string {
  switch (name) {
    case "congrats":
      return "\u{1F389}"; // 🎉
    case "like":
      return "\u{1F44D}"; // 👍
    case "love":
      return "\u{2764}\u{FE0F}"; // ❤️
    case "haha":
      return "\u{1F602}"; // 😂
    case "cheer":
      return "\u{1F4E3}"; // 📣
    case "high_five":
      return "\u{1F64C}"; // 🙌
    case "support":
      return "\u{1F91D}"; // 🤝
    default:
      return "\u{1F44D}";
  }
}

/** Reaction counts sorted highest-first, e.g. "🎉 45  ❤️ 2". */
export function reactionsSummary(card: FeedCard): string {
  // reactionCounts is absent entirely on cards that can't be reacted to (e.g. FOLLOW) --
  // the type says Record<string, number>, but that's not enforced at runtime, so guard it.
  return Object.entries(card.reactionCounts ?? {})
    .sort((a, b) => b[1] - a[1])
    .map(([name, count]) => `${reactionEmoji(name)} ${count}`)
    .join("  ");
}

/**
 * null when there's nothing to show; otherwise a "💬 N" or, if the preview's shape
 * doesn't match any known count field (unconfirmed, since no observed card has had
 * comments yet), a bare "💬" indicator.
 */
export function commentIndicator(card: FeedCard): string | null {
  const value = card.commentPreview;
  if (value === null || value === undefined) return null;
  if (typeof value !== "object") return "\u{1F4AC}";

  const obj = value as Record<string, unknown>;
  const count = obj.totalComments ?? obj.commentCount ?? obj.count;
  return typeof count === "number" ? `\u{1F4AC} ${count}` : "\u{1F4AC}";
}

/**
 * Turns a raw `picture` URL from the API into one that actually loads. Two issues found
 * by hand (the CDN returned no body on failure, just a bare HTTP status, so there was
 * nothing to inspect except by trying requests): URLs come back protocol-relative
 * (`//simg-ssl.duolingo.com/...`), and Duolingo's avatar CDN requires a trailing size
 * segment on *every* avatar URL, custom-rendered or default -- omitting it returns 400 on
 * the render-CDN host and 403 on the default-avatar host. Verified live: appending
 * `/medium` turned both of those into 200s.
 */
export function avatarUrl(url: string): string {
  const withScheme = url.startsWith("//") ? `https:${url}` : url;
  return `${withScheme.replace(/\/+$/, "")}/medium`;
}

export interface BadgeColor {
  r: number;
  g: number;
  b: number;
}

/**
 * A small badge to draw over/next to a feed card's avatar, derived from `kudosIcon`. Not
 * real Duolingo artwork (we never found a confirmed icon-image URL) -- just a glyph+color
 * standing in for it, consistent across every frontend (Rust included).
 */
export interface Badge {
  glyph: string;
  color: BadgeColor;
}

const GOLD: BadgeColor = { r: 0xe8, g: 0xb4, b: 0x23 };
const SILVER: BadgeColor = { r: 0xb0, g: 0xb8, b: 0xc1 };
const BRONZE: BadgeColor = { r: 0xc9, g: 0x7a, b: 0x3f };
const NEUTRAL: BadgeColor = { r: 0x6b, g: 0x7a, b: 0x88 };

/** null when the card has no badge-worthy `kudosIcon` (e.g. plain FOLLOW cards). */
export function badge(card: FeedCard): Badge | null {
  const icon = card.kudosIcon;
  if (!icon) return null;

  const rankPrefix = "kudos_feed_leaderboard_rank_";
  if (icon.startsWith(rankPrefix)) {
    const rank = icon.slice(rankPrefix.length);
    const color = rank === "1" ? GOLD : rank === "2" ? SILVER : rank === "3" ? BRONZE : NEUTRAL;
    return { glyph: rank, color };
  }
  if (icon.includes("league_promotion")) return { glyph: "\u{2B06}", color: GOLD }; // ⬆
  if (icon.includes("streak_milestone")) return { glyph: "\u{1F525}", color: BRONZE }; // 🔥
  if (icon.includes("sharpshooter")) return { glyph: "\u{1F3AF}", color: NEUTRAL }; // 🎯
  if (icon.includes("resurrection")) return { glyph: "\u{1F44B}", color: NEUTRAL }; // 👋
  if (icon.includes("friends_quest")) return { glyph: "\u{1F91D}", color: NEUTRAL }; // 🤝
  return { glyph: "\u{2B50}", color: NEUTRAL }; // ⭐
}

export function humanizeTimestamp(ts: number): string {
  const then = new Date(ts * 1000);
  const diffSeconds = Math.floor((Date.now() - then.getTime()) / 1000);

  if (diffSeconds < 60) return "just now";
  if (diffSeconds < 3600) return `${Math.floor(diffSeconds / 60)}m ago`;
  if (diffSeconds < 86400) return `${Math.floor(diffSeconds / 3600)}h ago`;
  if (diffSeconds < 30 * 86400) return `${Math.floor(diffSeconds / 86400)}d ago`;
  return then.toISOString().slice(0, 10);
}
