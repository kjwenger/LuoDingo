mod api;

use api::{Client, Friend};
use eframe::egui;

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Following,
    Followers,
}

struct App {
    client: Client,
    tab: Tab,
    following: Vec<Friend>,
    followers: Vec<Friend>,
    error: Option<String>,
}

impl App {
    fn new(client: Client) -> Self {
        let mut app = Self {
            client,
            tab: Tab::Following,
            following: Vec::new(),
            followers: Vec::new(),
            error: None,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.error = None;

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

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let panel_frame = egui::Frame::central_panel(&ui.style().clone());
        panel_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Duolingo Friend Updates");
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

            let friends = match self.tab {
                Tab::Following => &self.following,
                Tab::Followers => &self.followers,
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                Self::friends_table(ui, friends);
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
