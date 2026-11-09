use log::{debug, error, info, trace, warn};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;

const API_BASE: &str = "https://www.duolingo.com/2017-06-30";
const FRIENDS_API_BASE: &str = "https://www.duolingo.com/2023-05-23";

/// The reaction value the real app's "Celebrate" (FEIERN) button sends. Verified live:
/// posting this to a zero-reaction card made it show up as `reactionCounts: {"congrats": 1}`.
pub const CELEBRATE_REACTION: &str = "CONGRATS";

/// Downloads raw bytes from a public URL (e.g. an avatar image). Unauthenticated — avatar
/// CDN assets don't need the JWT. Frontends that can't load images by URL themselves
/// (unlike egui's built-in loaders) decode these bytes with their own toolkit.
pub fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    trace!("GET {url} (bytes)");
    let response = reqwest::blocking::get(url).map_err(|e| {
        let msg = format!("request to {url} failed: {e}");
        warn!("{msg}");
        msg
    })?;

    let status = response.status();
    if !status.is_success() {
        let msg = format!("GET {url} returned HTTP {status}");
        warn!("{msg}");
        return Err(msg);
    }

    let bytes = response.bytes().map_err(|e| {
        let msg = format!("failed to read response body from {url}: {e}");
        warn!("{msg}");
        msg
    })?;
    debug!("GET {url} -> {status}, {} bytes", bytes.len());
    Ok(bytes.to_vec())
}

#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    pub username: String,
    #[serde(rename = "totalXp")]
    pub total_xp: u64,
    pub streak: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Friend {
    pub id: u64,
    pub name: Option<String>,
    pub username: String,
    #[serde(rename = "totalXp")]
    pub total_xp: u64,
    #[serde(rename = "weeklyXp")]
    pub weekly_xp: u64,
    #[serde(rename = "monthlyXp")]
    pub monthly_xp: u64,
    #[serde(rename = "hasPlus")]
    pub has_plus: bool,
    #[serde(rename = "hasRecentActivity15")]
    pub has_recent_activity: bool,
    #[serde(default)]
    pub picture: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SubscriptionsResponse {
    subscriptions: Vec<Friend>,
}

#[derive(Debug, Deserialize)]
struct SubscribersResponse {
    subscribers: Vec<Friend>,
}

/// A single entry in the friends activity feed, e.g. a league promotion or a kudos offer.
#[derive(Debug, Deserialize, Clone)]
pub struct FeedCard {
    #[serde(rename = "cardType")]
    pub card_type: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub body: Option<String>,
    pub timestamp: i64,
    #[serde(rename = "userId")]
    pub user_id: u64,
    /// The friend's avatar image URL, when they've customized one.
    #[serde(default)]
    pub picture: Option<String>,
    /// Identifies which badge/icon the real app shows for this card, e.g.
    /// "kudos_feed_leaderboard_rank_1" or "kudos_feed_league_promotion_OBSIDIAN".
    /// Not an image URL itself — see `format::badge`.
    #[serde(rename = "kudosIcon", default)]
    pub kudos_icon: Option<String>,
    /// Identifies this specific event; required as `groupId` when reacting to the card
    /// via `Client::react`. Absent on cards that can't be reacted to (e.g. FOLLOW).
    #[serde(rename = "eventId", default)]
    pub event_id: Option<String>,
    /// Reaction name (e.g. "congrats", "love") -> count. Absent on cards that can't be
    /// reacted to (e.g. FOLLOW), and possibly empty on ones that can but haven't been yet.
    #[serde(rename = "reactionCounts", default)]
    pub reaction_counts: BTreeMap<String, u64>,
    /// The reaction *this* account has already left on the card, if any (lowercase, e.g.
    /// "congrats"). Used to show the Celebrate button as already-pressed.
    #[serde(rename = "reactionType", default)]
    pub reaction_type: Option<String>,
    /// Present (possibly null) on kudos-style cards. Its non-null shape is unconfirmed, since
    /// no card observed so far has had any comments.
    #[serde(rename = "commentPreview", default)]
    pub comment_preview: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct FeedResponse {
    feed: Vec<FeedGroup>,
}

#[derive(Debug, Deserialize)]
struct FeedGroup {
    #[serde(rename = "feedCards")]
    feed_cards: Vec<FeedCard>,
}

pub struct Client {
    http: reqwest::blocking::Client,
    jwt: String,
    user_id: u64,
}

impl Client {
    pub fn from_env() -> Result<Self, String> {
        let env_path = dotenvy::dotenv().map_err(|e| {
            let msg = format!("failed to load .env: {e}");
            error!("{msg}");
            msg
        })?;
        debug!("loaded .env from {}", env_path.display());

        let jwt = env::var("DUOLINGO_JWT_TOKEN").map_err(|_| {
            let msg = "DUOLINGO_JWT_TOKEN is not set in .env".to_string();
            error!("{msg}");
            msg
        })?;
        let user_id = user_id_from_jwt(&jwt)?;
        info!("Duolingo client ready (user_id={user_id})");

        Ok(Self {
            http: reqwest::blocking::Client::new(),
            jwt,
            user_id,
        })
    }

    pub fn profile(&self) -> Result<Profile, String> {
        self.get(&format!("{API_BASE}/users/{}", self.user_id))
    }

    /// People the logged-in user follows.
    pub fn following(&self) -> Result<Vec<Friend>, String> {
        let resp: SubscriptionsResponse =
            self.get(&format!("{API_BASE}/users/{}/subscriptions", self.user_id))?;
        info!("fetched {} following", resp.subscriptions.len());
        Ok(resp.subscriptions)
    }

    /// People who follow the logged-in user.
    pub fn followers(&self) -> Result<Vec<Friend>, String> {
        let resp: SubscribersResponse =
            self.get(&format!("{API_BASE}/users/{}/subscribers", self.user_id))?;
        info!("fetched {} followers", resp.subscribers.len());
        Ok(resp.subscribers)
    }

    /// The friends activity feed: league promotions, kudos, follows, etc.,
    /// most recent first.
    pub fn feed(&self) -> Result<Vec<FeedCard>, String> {
        let resp: FeedResponse = self.get(&format!(
            "{FRIENDS_API_BASE}/friends/users/{}/feed/v2?uiLanguage=en",
            self.user_id
        ))?;
        let mut cards: Vec<FeedCard> = resp.feed.into_iter().flat_map(|g| g.feed_cards).collect();
        cards.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        info!("fetched {} feed cards", cards.len());
        for card in &cards {
            trace!(
                "feed card: type={} event_id={:?} kudos_icon={:?} picture={:?}",
                card.card_type, card.event_id, card.kudos_icon, card.picture
            );
        }
        Ok(cards)
    }

    /// React to a feed card (e.g. the real app's "Celebrate" button sends `"CONGRATS"`).
    /// Verified live against the real API: a zero-reaction card went from no `congrats`
    /// entry to `{"congrats": 1}` after calling this with `reaction = "CONGRATS"`.
    pub fn react(&self, event_id: &str, reaction: &str) -> Result<(), String> {
        let url = "https://www.duolingo.com/card/reaction";
        let body = serde_json::json!({
            "groupId": event_id,
            "reaction": reaction,
            "trackingProperties": { "screen": "kudos_feed" },
            "userId": self.user_id,
        });

        info!("POST {url} (reaction={reaction}, event_id={event_id})");
        let response = self.http.post(url).bearer_auth(&self.jwt).json(&body).send().map_err(|e| {
            let msg = format!("request to {url} failed: {e}");
            error!("{msg}");
            msg
        })?;

        let status = response.status();
        if !status.is_success() {
            let msg = format!("Duolingo API returned HTTP {status} for {url}");
            warn!("{msg}");
            return Err(msg);
        }
        debug!("reaction accepted: {status}");
        Ok(())
    }

    fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, String> {
        debug!("GET {url}");
        let response = self.http.get(url).bearer_auth(&self.jwt).send().map_err(|e| {
            let msg = format!("request to {url} failed: {e}");
            error!("{msg}");
            msg
        })?;

        let status = response.status();
        if !status.is_success() {
            let msg = format!("Duolingo API returned HTTP {status} for {url}");
            warn!("{msg}");
            return Err(msg);
        }
        trace!("GET {url} -> {status}");

        response.json::<T>().map_err(|e| {
            let msg = format!("failed to parse response from {url}: {e}");
            error!("{msg}");
            msg
        })
    }
}

fn user_id_from_jwt(jwt: &str) -> Result<u64, String> {
    let payload = jwt
        .split('.')
        .nth(1)
        .ok_or("JWT is missing its payload segment")?;

    use base64::Engine;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload.trim_end_matches('='))
        .map_err(|e| format!("failed to base64-decode JWT payload: {e}"))?;

    let claims: serde_json::Value =
        serde_json::from_slice(&decoded).map_err(|e| format!("JWT payload is not JSON: {e}"))?;

    claims["sub"]
        .as_u64()
        .ok_or_else(|| "JWT payload has no numeric 'sub' claim".to_string())
}
