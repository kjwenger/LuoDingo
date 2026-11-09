// Mirrors the field set actually read in rust/core/src/api.rs -- not the full wire
// response, which has many more fields (see Profile/FeedCard in public/duolingo.yaml for
// the fuller, documented picture). TS/JSON needs no serde-style renaming, so these line
// up with the wire JSON field names directly.

export interface Profile {
  username: string;
  totalXp: number;
  streak: number | null;
}

export interface Friend {
  id: number;
  name: string | null;
  username: string;
  totalXp: number;
  weeklyXp: number;
  monthlyXp: number;
  hasPlus: boolean;
  hasRecentActivity15: boolean;
  picture: string | null;
}

export interface FeedCard {
  cardType: string;
  displayName: string | null;
  body: string | null;
  timestamp: number;
  userId: number;
  picture: string | null;
  kudosIcon: string | null;
  eventId: string | null;
  reactionCounts: Record<string, number>;
  reactionType: string | null;
  commentPreview: unknown;
  // SHARE_SENTENCE_OFFER-only: body is "" for this card type, the actual content lives
  // here instead.
  fromSentence: string | null;
  toSentence: string | null;
}
