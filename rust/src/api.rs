use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;

const API_BASE: &str = "https://www.duolingo.com/2017-06-30";
const FRIENDS_API_BASE: &str = "https://www.duolingo.com/2023-05-23";

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
    /// Reaction name (e.g. "congrats", "love") -> count. Absent on cards that can't be
    /// reacted to (e.g. FOLLOW), and possibly empty on ones that can but haven't been yet.
    #[serde(rename = "reactionCounts", default)]
    pub reaction_counts: BTreeMap<String, u64>,
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
        dotenvy::dotenv().map_err(|e| format!("failed to load .env: {e}"))?;

        let jwt = env::var("DUOLINGO_JWT_TOKEN")
            .map_err(|_| "DUOLINGO_JWT_TOKEN is not set in .env".to_string())?;
        let user_id = user_id_from_jwt(&jwt)?;

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
        Ok(resp.subscriptions)
    }

    /// People who follow the logged-in user.
    pub fn followers(&self) -> Result<Vec<Friend>, String> {
        let resp: SubscribersResponse =
            self.get(&format!("{API_BASE}/users/{}/subscribers", self.user_id))?;
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
        Ok(cards)
    }

    fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, String> {
        let response = self
            .http
            .get(url)
            .bearer_auth(&self.jwt)
            .send()
            .map_err(|e| format!("request to {url} failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Duolingo API returned HTTP {} for {url}", response.status()));
        }

        response
            .json::<T>()
            .map_err(|e| format!("failed to parse response from {url}: {e}"))
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
