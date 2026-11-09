//! Pure formatting helpers shared by every frontend. No UI-toolkit types here —
//! just plain `String`s so egui, libadwaita, or anything else can render them as-is.

use crate::api::FeedCard;

/// The card's displayable achievement/update text. Most card types already have this in
/// `body`; SHARE_SENTENCE_OFFER is the one confirmed exception -- `body` is `""` there,
/// with the actual content in `from_sentence`/`to_sentence` instead. Found live (via the
/// Next.js frontend first): a raw card showed the literal string "SHARE_SENTENCE_OFFER"
/// to the user before this existed, because the old fallback (`body` -> `card_type`) had
/// nothing better to fall back to.
pub fn card_body(card: &FeedCard) -> String {
    if card.card_type == "SHARE_SENTENCE_OFFER"
        && let (Some(from), Some(to)) = (&card.from_sentence, &card.to_sentence)
    {
        return format!("\"{from}\" \u{2192} \"{to}\"");
    }
    card.body
        .clone()
        .unwrap_or_else(|| card.card_type.clone())
}

/// Whether the Celebrate button should be shown at all. `event_id` alone isn't enough --
/// found live: FOLLOW cards have a real event_id but `reaction_counts` comes back `null`
/// (empty map here, since the field defaults), because Duolingo tracks no reaction
/// mechanism for them at all. SHARE_SENTENCE_OFFER cards use `like`/`haha`/`love`
/// reactions, not `congrats`. Sending CONGRATS (the only reaction value actually verified
/// against the live API, see `api::CELEBRATE_REACTION`) would be wrong for either. Only
/// show it where CONGRATS is the reaction the real app itself would send for a bare click.
pub fn can_celebrate(card: &FeedCard) -> bool {
    card.event_id.is_some()
        && matches!(card.card_type.as_str(), "KUDOS_OFFER" | "KUDOS_MILESTONE")
}

pub fn reaction_emoji(name: &str) -> &'static str {
    match name {
        "congrats" => "\u{1F389}",      // 🎉
        "like" => "\u{1F44D}",          // 👍
        "love" => "\u{2764}\u{FE0F}",   // ❤️
        "haha" => "\u{1F602}",          // 😂
        "cheer" => "\u{1F4E3}",         // 📣
        "high_five" => "\u{1F64C}",     // 🙌
        "support" => "\u{1F91D}",       // 🤝
        _ => "\u{1F44D}",
    }
}

/// Reaction counts sorted highest-first, e.g. "🎉 45  ❤️ 2".
pub fn reactions_summary(card: &FeedCard) -> String {
    let mut counts: Vec<(&String, &u64)> = card.reaction_counts.iter().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1));
    counts
        .into_iter()
        .map(|(name, count)| format!("{} {count}", reaction_emoji(name)))
        .collect::<Vec<_>>()
        .join("  ")
}

/// None when there's nothing to show; otherwise a "💬 N" or, if the preview's shape doesn't
/// match any known count field (unconfirmed, since no observed card has had comments yet),
/// a bare "💬" indicator.
pub fn comment_indicator(card: &FeedCard) -> Option<String> {
    let value = card.comment_preview.as_ref()?;
    if value.is_null() {
        return None;
    }
    let count = value
        .get("totalComments")
        .or_else(|| value.get("commentCount"))
        .or_else(|| value.get("count"))
        .and_then(|v| v.as_u64());

    Some(match count {
        Some(n) => format!("\u{1F4AC} {n}"),
        None => "\u{1F4AC}".to_string(),
    })
}

/// Turns a raw `picture` URL from the API into one that actually loads. Two issues found by
/// hand (the CDN returned no body on failure, just a bare HTTP status, so there was nothing
/// to inspect except by trying requests): URLs come back protocol-relative
/// (`//simg-ssl.duolingo.com/...`), and Duolingo's avatar CDN requires a trailing size
/// segment on *every* avatar URL, custom-rendered or default — omitting it returns 400 on
/// the render-CDN host and 403 on the default-avatar host. Verified live: appending
/// `/medium` turned both of those into 200s.
pub fn avatar_url(url: &str) -> String {
    let url = match url.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => url.to_string(),
    };
    format!("{}/medium", url.trim_end_matches('/'))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BadgeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// A small badge to draw over/next to a feed card's avatar, derived from `kudosIcon`.
/// Not real Duolingo artwork (we never found a confirmed icon-image URL) — just a
/// glyph+color standing in for it, consistent across every frontend.
#[derive(Debug, Clone)]
pub struct Badge {
    pub glyph: String,
    pub color: BadgeColor,
}

const GOLD: BadgeColor = BadgeColor { r: 0xE8, g: 0xB4, b: 0x23 };
const SILVER: BadgeColor = BadgeColor { r: 0xB0, g: 0xB8, b: 0xC1 };
const BRONZE: BadgeColor = BadgeColor { r: 0xC9, g: 0x7A, b: 0x3F };
const NEUTRAL: BadgeColor = BadgeColor { r: 0x6B, g: 0x7A, b: 0x88 };

/// None when the card has no badge-worthy `kudosIcon` (e.g. plain FOLLOW cards).
pub fn badge(card: &FeedCard) -> Option<Badge> {
    let icon = card.kudos_icon.as_deref()?;

    Some(if let Some(rank) = icon.strip_prefix("kudos_feed_leaderboard_rank_") {
        let color = match rank {
            "1" => GOLD,
            "2" => SILVER,
            "3" => BRONZE,
            _ => NEUTRAL,
        };
        Badge { glyph: rank.to_string(), color }
    } else if icon.contains("league_promotion") {
        Badge { glyph: "\u{2B06}".to_string(), color: GOLD } // ⬆
    } else if icon.contains("streak_milestone") {
        Badge { glyph: "\u{1F525}".to_string(), color: BRONZE } // 🔥
    } else if icon.contains("sharpshooter") {
        Badge { glyph: "\u{1F3AF}".to_string(), color: NEUTRAL } // 🎯
    } else if icon.contains("resurrection") {
        Badge { glyph: "\u{1F44B}".to_string(), color: NEUTRAL } // 👋
    } else if icon.contains("friends_quest") {
        Badge { glyph: "\u{1F91D}".to_string(), color: NEUTRAL } // 🤝
    } else {
        Badge { glyph: "\u{2B50}".to_string(), color: NEUTRAL } // ⭐
    })
}

pub fn humanize_timestamp(ts: i64) -> String {
    use chrono::{Local, TimeZone};

    let then = match Local.timestamp_opt(ts, 0) {
        chrono::offset::LocalResult::Single(dt) => dt,
        _ => return "unknown time".to_string(),
    };
    let diff = Local::now().signed_duration_since(then);

    if diff.num_seconds() < 60 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else if diff.num_days() < 30 {
        format!("{}d ago", diff.num_days())
    } else {
        then.format("%Y-%m-%d").to_string()
    }
}
