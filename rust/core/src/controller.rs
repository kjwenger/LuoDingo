//! Fetches and holds the app's data. Every frontend owns one `Controller` and
//! reads its fields; none of this depends on any particular UI toolkit.

use crate::api::{Client, FeedCard, Friend, CELEBRATE_REACTION};
use log::{info, warn};

pub struct Controller {
    client: Client,
    pub updates: Vec<FeedCard>,
    pub following: Vec<Friend>,
    pub followers: Vec<Friend>,
    pub error: Option<String>,
}

impl Controller {
    pub fn new() -> Result<Self, String> {
        let client = Client::from_env()?;
        let mut controller = Self {
            client,
            updates: Vec::new(),
            following: Vec::new(),
            followers: Vec::new(),
            error: None,
        };
        controller.refresh();
        Ok(controller)
    }

    pub fn refresh(&mut self) {
        info!("refresh: starting");
        self.error = None;

        match self.client.feed() {
            Ok(cards) => self.updates = cards,
            Err(e) => {
                warn!("refresh: feed failed: {e}");
                self.error = Some(format!("feed: {e}"));
            }
        }

        match self.client.following() {
            Ok(mut friends) => {
                friends.sort_by(|a, b| b.weekly_xp.cmp(&a.weekly_xp));
                self.following = friends;
            }
            Err(e) => {
                warn!("refresh: following failed: {e}");
                self.error = Some(format!("following: {e}"));
            }
        }

        match self.client.followers() {
            Ok(mut friends) => {
                friends.sort_by(|a, b| b.weekly_xp.cmp(&a.weekly_xp));
                self.followers = friends;
            }
            Err(e) => {
                warn!("refresh: followers failed: {e}");
                self.error = Some(format!("followers: {e}"));
            }
        }

        info!(
            "refresh: done ({} updates, {} following, {} followers, error={:?})",
            self.updates.len(),
            self.following.len(),
            self.followers.len(),
            self.error
        );
    }

    /// Sends a "Celebrate" reaction for the given update and, on success, updates that
    /// card's counts locally so the UI reflects it immediately without a full refresh.
    pub fn celebrate(&mut self, event_id: &str) {
        info!("celebrate: event_id={event_id}");
        match self.client.react(event_id, CELEBRATE_REACTION) {
            Ok(()) => {
                if let Some(card) = self
                    .updates
                    .iter_mut()
                    .find(|c| c.event_id.as_deref() == Some(event_id))
                {
                    *card.reaction_counts.entry("congrats".to_string()).or_insert(0) += 1;
                    card.reaction_type = Some("congrats".to_string());
                } else {
                    warn!("celebrate: event_id={event_id} reacted but no matching card found locally");
                }
            }
            Err(e) => {
                warn!("celebrate: event_id={event_id} failed: {e}");
                self.error = Some(format!("celebrate: {e}"));
            }
        }
    }
}
