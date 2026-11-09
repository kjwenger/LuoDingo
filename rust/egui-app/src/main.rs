use eframe::egui;
use luodingo_core::format::{comment_indicator, humanize_timestamp, reactions_summary};
use luodingo_core::{Controller, FeedCard, Friend};

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Updates,
    Following,
    Followers,
}

struct App {
    controller: Controller,
    tab: Tab,
}

impl App {
    fn new(controller: Controller) -> Self {
        Self {
            controller,
            tab: Tab::Updates,
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

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let panel_frame = egui::Frame::central_panel(&ui.style().clone());
        panel_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("LuoDingo");
                if ui.button("Refresh").clicked() {
                    self.controller.refresh();
                }
            });

            if let Some(err) = &self.controller.error {
                ui.colored_label(egui::Color32::RED, err);
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.tab,
                    Tab::Updates,
                    format!("Updates ({})", self.controller.updates.len()),
                );
                ui.selectable_value(
                    &mut self.tab,
                    Tab::Following,
                    format!("Following ({})", self.controller.following.len()),
                );
                ui.selectable_value(
                    &mut self.tab,
                    Tab::Followers,
                    format!("Followers ({})", self.controller.followers.len()),
                );
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                Tab::Updates => Self::updates_list(ui, &self.controller.updates),
                Tab::Following => Self::friends_table(ui, &self.controller.following),
                Tab::Followers => Self::friends_table(ui, &self.controller.followers),
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let controller = match Controller::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    eframe::run_native(
        "LuoDingo - Friend Updates",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App::new(controller)))),
    )
}
