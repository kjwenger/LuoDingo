mod api;

use api::{Client, FeedCard, Friend};
use eframe::egui;

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Updates,
    Following,
    Followers,
}

struct App {
    client: Client,
    tab: Tab,
    updates: Vec<FeedCard>,
    following: Vec<Friend>,
    followers: Vec<Friend>,
    error: Option<String>,
}

impl App {
    fn new(client: Client) -> Self {
        let mut app = Self {
            client,
            tab: Tab::Updates,
            updates: Vec::new(),
            following: Vec::new(),
            followers: Vec::new(),
            error: None,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.error = None;

        match self.client.feed() {
            Ok(cards) => self.updates = cards,
            Err(e) => self.error = Some(format!("feed: {e}")),
        }

        match self.client.following() {
            Ok(mut friends) => {
                friends.sort_by(|a, b| b.weekly_xp.cmp(&a.weekly_xp));
                self.following = friends;
            }
            Err(e) => self.error = Some(format!("following: {e}")),
        }

        match self.client.followers() {
            Ok(mut friends) => {
                friends.sort_by(|a, b| b.weekly_xp.cmp(&a.weekly_xp));
                self.followers = friends;
            }
            Err(e) => self.error = Some(format!("followers: {e}")),
        }
    }

    fn updates_list(ui: &mut egui::Ui, cards: &[FeedCard]) {
        if cards.is_empty() {
            ui.label("No recent friend updates.");
            return;
        }

        for card in cards {
            ui.horizontal_wrapped(|ui| {
                let name = card.display_name.as_deref().unwrap_or("Someone");
                ui.strong(name);
                ui.label(card.body.as_deref().unwrap_or(card.card_type.as_str()));
            });

            ui.horizontal(|ui| {
                let meta = egui::RichText::new(humanize_timestamp(card.timestamp))
                    .small()
                    .color(egui::Color32::GRAY);
                ui.label(meta);

                if !card.reaction_counts.is_empty() {
                    let text = egui::RichText::new(format!("· {}", reactions_summary(card)))
                        .small()
                        .color(egui::Color32::GRAY);
                    ui.label(text);
                }

                if let Some(comments) = comment_indicator(card) {
                    let text = egui::RichText::new(format!("· {comments}"))
                        .small()
                        .color(egui::Color32::GRAY);
                    ui.label(text);
                }
            });

            ui.separator();
        }
    }

    fn friends_table(ui: &mut egui::Ui, friends: &[Friend]) {
        use egui_extras::{Column, TableBuilder};

        TableBuilder::new(ui)
            .striped(true)
            .column(Column::remainder().at_least(140.0))
            .column(Column::auto().at_least(120.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::auto().at_least(60.0))
            .header(22.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Username");
                });
                header.col(|ui| {
                    ui.strong("Total XP");
                });
                header.col(|ui| {
                    ui.strong("Weekly XP");
                });
                header.col(|ui| {
                    ui.strong("Monthly XP");
                });
                header.col(|ui| {
                    ui.strong("Active");
                });
            })
            .body(|body| {
                body.rows(20.0, friends.len(), |mut row| {
                    let friend = &friends[row.index()];
                    let display_name = friend.name.as_deref().unwrap_or(&friend.username);
                    row.col(|ui| {
                        ui.label(if friend.has_plus {
                            format!("{display_name} \u{1F48E}")
                        } else {
                            display_name.to_string()
                        });
                    });
                    row.col(|ui| {
                        ui.label(&friend.username);
                    });
                    row.col(|ui| {
                        ui.label(friend.total_xp.to_string());
                    });
                    row.col(|ui| {
                        ui.label(friend.weekly_xp.to_string());
                    });
                    row.col(|ui| {
                        ui.label(friend.monthly_xp.to_string());
                    });
                    row.col(|ui| {
                        ui.label(if friend.has_recent_activity { "\u{1F7E2}" } else { "" });
                    });
                });
            });
    }
}

fn reaction_emoji(name: &str) -> &'static str {
    match name {
        "congrats" => "\u{1F389}",  // 🎉
        "like" => "\u{1F44D}",      // 👍
        "love" => "\u{2764}\u{FE0F}", // ❤️
        "haha" => "\u{1F602}",      // 😂
        "cheer" => "\u{1F4E3}",     // 📣
        "high_five" => "\u{1F64C}", // 🙌
        "support" => "\u{1F91D}",  // 🤝
        _ => "\u{1F44D}",
    }
}

/// Reaction counts sorted highest-first, e.g. "🎉 45  ❤️ 2".
fn reactions_summary(card: &FeedCard) -> String {
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
fn comment_indicator(card: &FeedCard) -> Option<String> {
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

fn humanize_timestamp(ts: i64) -> String {
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

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let panel_frame = egui::Frame::central_panel(&ui.style().clone());
        panel_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("LuoDingo");
                if ui.button("Refresh").clicked() {
                    self.refresh();
                }
            });

            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::RED, err);
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.tab,
                    Tab::Updates,
                    format!("Updates ({})", self.updates.len()),
                );
                ui.selectable_value(
                    &mut self.tab,
                    Tab::Following,
                    format!("Following ({})", self.following.len()),
                );
                ui.selectable_value(
                    &mut self.tab,
                    Tab::Followers,
                    format!("Followers ({})", self.followers.len()),
                );
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                Tab::Updates => Self::updates_list(ui, &self.updates),
                Tab::Following => Self::friends_table(ui, &self.following),
                Tab::Followers => Self::friends_table(ui, &self.followers),
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let client = match Client::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    eframe::run_native(
        "LuoDingo - Friend Updates",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App::new(client)))),
    )
}
