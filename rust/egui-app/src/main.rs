use eframe::egui;
use luodingo_core::format::{
    self, comment_indicator, humanize_timestamp, normalize_avatar_url, reactions_summary,
};
use luodingo_core::{Controller, FeedCard, Friend};

// Approximate Duolingo dark-theme palette (not pixel-matched to the real site).
const BG: egui::Color32 = egui::Color32::from_rgb(0x13, 0x1F, 0x24);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(0x1F, 0x2D, 0x35);
const ACCENT_GREEN: egui::Color32 = egui::Color32::from_rgb(0x58, 0xCC, 0x02);
const MUTED: egui::Color32 = egui::Color32::from_gray(0x8A);

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

    fn avatar(ui: &mut egui::Ui, picture: Option<&str>, size: f32) {
        if let Some(url) = picture {
            ui.add(
                egui::Image::new(normalize_avatar_url(url))
                    .fit_to_exact_size(egui::vec2(size, size))
                    .corner_radius((size / 2.0) as u8),
            );
        } else {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), size / 2.0, MUTED);
        }
    }

    fn badge(ui: &mut egui::Ui, badge: &format::Badge, size: f32) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
        let color = egui::Color32::from_rgb(badge.color.r, badge.color.g, badge.color.b);
        ui.painter().circle_filled(rect.center(), size / 2.0, color);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &badge.glyph,
            egui::FontId::proportional(size * 0.5),
            egui::Color32::BLACK,
        );
    }

    /// Renders the Updates tab as cards; returns the event id to celebrate, if the
    /// Celebrate button was clicked this frame.
    fn updates_list(ui: &mut egui::Ui, cards: &[FeedCard]) -> Option<String> {
        if cards.is_empty() {
            ui.label("No recent friend updates.");
            return None;
        }

        let mut to_celebrate = None;

        for card in cards {
            egui::Frame::new()
                .fill(CARD_BG)
                .corner_radius(10)
                .inner_margin(12.0)
                .outer_margin(egui::Margin::symmetric(0, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        Self::avatar(ui, card.picture.as_deref(), 44.0);

                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 48.0);

                            ui.horizontal(|ui| {
                                let name = card.display_name.as_deref().unwrap_or("Someone");
                                ui.strong(name);
                                ui.label(
                                    egui::RichText::new(humanize_timestamp(card.timestamp))
                                        .small()
                                        .color(MUTED),
                                );

                                if let Some(b) = format::badge(card) {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| Self::badge(ui, &b, 24.0),
                                    );
                                }
                            });

                            ui.label(card.body.as_deref().unwrap_or(card.card_type.as_str()));

                            ui.horizontal(|ui| {
                                if let Some(event_id) = &card.event_id {
                                    let already = card.reaction_type.is_some();
                                    let label = if already {
                                        "\u{1F389} Celebrated"
                                    } else {
                                        "\u{1F389} Celebrate"
                                    };
                                    if ui.add_enabled(!already, egui::Button::new(label)).clicked()
                                    {
                                        to_celebrate = Some(event_id.clone());
                                    }
                                }

                                if !card.reaction_counts.is_empty() {
                                    ui.label(
                                        egui::RichText::new(reactions_summary(card))
                                            .small()
                                            .color(MUTED),
                                    );
                                }

                                if let Some(comments) = comment_indicator(card) {
                                    ui.label(egui::RichText::new(comments).small().color(MUTED));
                                }
                            });
                        });
                    });
                });
        }

        to_celebrate
    }

    fn friends_table(ui: &mut egui::Ui, friends: &[Friend]) {
        use egui_extras::{Column, TableBuilder};

        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto().at_least(36.0))
            .column(Column::remainder().at_least(140.0))
            .column(Column::auto().at_least(120.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::auto().at_least(60.0))
            .header(22.0, |mut header| {
                header.col(|_ui| {});
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
                body.rows(32.0, friends.len(), |mut row| {
                    let friend = &friends[row.index()];
                    let display_name = friend.name.as_deref().unwrap_or(&friend.username);
                    row.col(|ui| {
                        Self::avatar(ui, friend.picture.as_deref(), 28.0);
                    });
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
        let panel_frame = egui::Frame::central_panel(&ui.style().clone()).fill(BG);
        panel_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("LuoDingo").color(ACCENT_GREEN));
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

            let mut to_celebrate = None;
            egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                Tab::Updates => to_celebrate = Self::updates_list(ui, &self.controller.updates),
                Tab::Following => Self::friends_table(ui, &self.controller.following),
                Tab::Followers => Self::friends_table(ui, &self.controller.followers),
            });

            if let Some(event_id) = to_celebrate {
                self.controller.celebrate(&event_id);
            }
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
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(App::new(controller)))
        }),
    )
}
