//! Pure formatting helpers shared by every frontend. No UI-toolkit types here —
//! just plain `String`s so egui, libadwaita, or anything else can render them as-is.

use crate::api::FeedCard;

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
