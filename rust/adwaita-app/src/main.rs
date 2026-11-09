use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use luodingo_core::format::{comment_indicator, humanize_timestamp, reactions_summary};
use luodingo_core::{Controller, FeedCard, Friend};

fn clear(list: &gtk::ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn update_row(card: &FeedCard) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(card.display_name.as_deref().unwrap_or("Someone"));

    let mut subtitle = format!(
        "{} · {}",
        card.body.as_deref().unwrap_or(card.card_type.as_str()),
        humanize_timestamp(card.timestamp)
    );
    if !card.reaction_counts.is_empty() {
        subtitle.push_str(&format!(" · {}", reactions_summary(card)));
    }
    if let Some(comments) = comment_indicator(card) {
        subtitle.push_str(&format!(" · {comments}"));
    }
    row.set_subtitle(&subtitle);
    row
}

fn friend_row(friend: &Friend) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    let name = friend.name.as_deref().unwrap_or(&friend.username);
    let display_name = if friend.has_plus {
        format!("{name} \u{1F48E}")
    } else {
        name.to_string()
    };
    row.set_title(&display_name);
    row.set_subtitle(&format!(
        "@{} · {} XP this week · {} total{}",
        friend.username,
        friend.weekly_xp,
        friend.total_xp,
        if friend.has_recent_activity { " · \u{1F7E2}" } else { "" }
    ));
    row
}

fn render(
    controller: &Controller,
    banner: &adw::Banner,
    updates_list: &gtk::ListBox,
    following_list: &gtk::ListBox,
    followers_list: &gtk::ListBox,
) {
    match &controller.error {
        Some(err) => {
            banner.set_title(err);
            banner.set_revealed(true);
        }
        None => banner.set_revealed(false),
    }

    clear(updates_list);
    if controller.updates.is_empty() {
        updates_list.append(&adw::ActionRow::builder().title("No recent friend updates.").build());
    }
    for card in &controller.updates {
        updates_list.append(&update_row(card));
    }

    clear(following_list);
    for friend in &controller.following {
        following_list.append(&friend_row(friend));
    }

    clear(followers_list);
    for friend in &controller.followers {
        followers_list.append(&friend_row(friend));
    }
}

fn scrolled_list() -> (gtk::ScrolledWindow, gtk::ListBox) {
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    let scrolled = gtk::ScrolledWindow::builder().child(&list).vexpand(true).build();
    (scrolled, list)
}

fn build_window(app: &adw::Application, controller: Controller) -> adw::ApplicationWindow {
    let controller = Rc::new(RefCell::new(controller));

    let (updates_scroller, updates_list) = scrolled_list();
    let (following_scroller, following_list) = scrolled_list();
    let (followers_scroller, followers_list) = scrolled_list();

    let stack = adw::ViewStack::new();
    stack.add_titled(&updates_scroller, Some("updates"), "Updates");
    stack.add_titled(&following_scroller, Some("following"), "Following");
    stack.add_titled(&followers_scroller, Some("followers"), "Followers");

    let switcher_title = adw::ViewSwitcherTitle::new();
    switcher_title.set_title("LuoDingo");
    switcher_title.set_stack(Some(&stack));

    let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh_button.set_tooltip_text(Some("Refresh"));

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&switcher_title));
    header.pack_end(&refresh_button);

    let banner = adw::Banner::new("");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&banner);
    content.append(&stack);

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&content));

    render(&controller.borrow(), &banner, &updates_list, &following_list, &followers_list);

    refresh_button.connect_clicked({
        let controller = controller.clone();
        let banner = banner.clone();
        move |_| {
            controller.borrow_mut().refresh();
            render(&controller.borrow(), &banner, &updates_list, &following_list, &followers_list);
        }
    });

    adw::ApplicationWindow::builder()
        .application(app)
        .title("LuoDingo")
        .default_width(700)
        .default_height(600)
        .content(&toolbar_view)
        .build()
}

fn main() -> gtk::glib::ExitCode {
    let controller = match Controller::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let app = adw::Application::builder()
        .application_id("io.github.kjwenger.LuoDingo")
        .build();

    let controller = RefCell::new(Some(controller));
    app.connect_activate(move |app| {
        let controller = controller.borrow_mut().take().expect("activate called once");
        build_window(app, controller).present();
    });

    app.run()
}
