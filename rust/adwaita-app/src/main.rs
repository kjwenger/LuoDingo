use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use log::{debug, info, warn};

use luodingo_core::format::{self, avatar_url, comment_indicator, humanize_timestamp, reactions_summary};
use luodingo_core::{fetch_bytes, Controller, FeedCard, Friend};

const CARD_CSS: &str = "
.luodingo-card {
    background-color: #1f2d35;
    border-radius: 10px;
    padding: 10px;
    margin: 4px 8px;
}
";

struct AppState {
    controller: Controller,
    avatar_cache: HashMap<String, gdk::Texture>,
}

#[derive(Clone)]
struct Widgets {
    banner: adw::Banner,
    celebrate_all_button: gtk::Button,
    updates_list: gtk::Box,
    following_list: gtk::ListBox,
    followers_list: gtk::ListBox,
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CARD_CSS);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Builds an avatar widget immediately (initials fallback, so there's always something to
/// show) and, unless the image is already cached, fetches+decodes it in the background so
/// building a long list of these never blocks the GTK main thread — the window would
/// otherwise never get a chance to actually present (see git history: it used to fetch
/// every avatar synchronously before the window existed, which for ~300 friends meant no
/// window appeared for the better part of a minute).
fn avatar_widget(url: Option<&str>, name: &str, state: &Rc<RefCell<AppState>>) -> adw::Avatar {
    let avatar = adw::Avatar::new(40, Some(name), true);

    let Some(raw_url) = url else {
        return avatar;
    };
    let url = avatar_url(raw_url);

    if let Some(texture) = state.borrow().avatar_cache.get(&url).cloned() {
        debug!("avatar cache hit: {url}");
        avatar.set_custom_image(Some(&texture));
        return avatar;
    }

    debug!("avatar cache miss, fetching async: {url}");
    let state = state.clone();
    let avatar_clone = avatar.clone();
    let fetch_url = url.clone();

    glib::MainContext::default().spawn_local(async move {
        let bytes = match gio::spawn_blocking(move || fetch_bytes(&fetch_url)).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => {
                warn!("avatar failed to fetch ({url}): {e}");
                return;
            }
            Err(e) => {
                warn!("avatar fetch task failed ({url}): {e:?}");
                return;
            }
        };

        match gdk::Texture::from_bytes(&glib::Bytes::from(&bytes)) {
            Ok(texture) => {
                debug!("avatar ready: {url} ({} bytes)", bytes.len());
                state.borrow_mut().avatar_cache.insert(url, texture.clone());
                avatar_clone.set_custom_image(Some(&texture));
            }
            Err(e) => warn!("avatar failed to decode ({url}, {} bytes): {e}", bytes.len()),
        }
    });

    avatar
}

fn badge_label(badge: &format::Badge) -> gtk::Label {
    let label = gtk::Label::new(None);
    label.set_markup(&format!(
        "<span background=\"#{:02X}{:02X}{:02X}\" foreground=\"black\">  {}  </span>",
        badge.color.r, badge.color.g, badge.color.b, badge.glyph
    ));
    label
}

fn update_card(
    card: &FeedCard,
    state: &Rc<RefCell<AppState>>,
    widgets: &Widgets,
) -> gtk::Box {
    let outer = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    outer.add_css_class("luodingo-card");

    let name = card.display_name.as_deref().unwrap_or("Someone").to_string();
    outer.append(&avatar_widget(card.picture.as_deref(), &name, state));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    content.set_hexpand(true);

    let header_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let name_label = gtk::Label::new(None);
    name_label.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(&name)));
    name_label.set_halign(gtk::Align::Start);
    header_row.append(&name_label);

    let time_label = gtk::Label::new(Some(&humanize_timestamp(card.timestamp)));
    time_label.add_css_class("dim-label");
    header_row.append(&time_label);

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    header_row.append(&spacer);

    if let Some(b) = format::badge(card) {
        header_row.append(&badge_label(&b));
    }
    content.append(&header_row);

    let body_text = card.body.clone().unwrap_or_else(|| card.card_type.clone());
    let body_label = gtk::Label::new(Some(&body_text));
    body_label.set_halign(gtk::Align::Start);
    body_label.set_wrap(true);
    body_label.set_xalign(0.0);
    content.append(&body_label);

    let actions_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    if let Some(event_id) = card.event_id.clone() {
        let already = card.reaction_type.is_some();
        let celebrate_btn =
            gtk::Button::with_label(if already { "\u{1F389} Celebrated" } else { "\u{1F389} Celebrate" });
        celebrate_btn.set_sensitive(!already);
        celebrate_btn.connect_clicked({
            let state = state.clone();
            let widgets = widgets.clone();
            move |_| {
                info!("Celebrate clicked: event_id={event_id}");
                state.borrow_mut().controller.celebrate(&event_id);
                render(&state, &widgets);
            }
        });
        actions_row.append(&celebrate_btn);
    }

    if !card.reaction_counts.is_empty() {
        let reactions_label = gtk::Label::new(Some(&reactions_summary(card)));
        reactions_label.add_css_class("dim-label");
        actions_row.append(&reactions_label);
    }

    if let Some(comments) = comment_indicator(card) {
        let comments_label = gtk::Label::new(Some(&comments));
        comments_label.add_css_class("dim-label");
        actions_row.append(&comments_label);
    }

    content.append(&actions_row);
    outer.append(&content);
    outer
}

fn friend_row(friend: &Friend, state: &Rc<RefCell<AppState>>) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    let name = friend.name.as_deref().unwrap_or(&friend.username);
    let display_name = if friend.has_plus {
        format!("{name} \u{1F48E}")
    } else {
        name.to_string()
    };
    row.set_title(&display_name);
    row.set_subtitle(&format!(
        "@{} \u{00B7} {} XP this week \u{00B7} {} total{}",
        friend.username,
        friend.weekly_xp,
        friend.total_xp,
        if friend.has_recent_activity { " \u{00B7} \u{1F7E2}" } else { "" }
    ));
    row.add_prefix(&avatar_widget(friend.picture.as_deref(), name, state));
    row
}

fn clear_listbox(list: &gtk::ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn render(state: &Rc<RefCell<AppState>>, widgets: &Widgets) {
    let (error, updates, following, followers, celebratable) = {
        let st = state.borrow();
        (
            st.controller.error.clone(),
            st.controller.updates.clone(),
            st.controller.following.clone(),
            st.controller.followers.clone(),
            st.controller.celebratable_count(),
        )
    };
    debug!(
        "render: {} updates, {} following, {} followers, {celebratable} celebratable, error={error:?}",
        updates.len(),
        following.len(),
        followers.len()
    );

    widgets
        .celebrate_all_button
        .set_label(&format!("\u{1F389} Celebrate All ({celebratable})"));
    widgets.celebrate_all_button.set_sensitive(celebratable > 0);

    match error {
        Some(err) => {
            widgets.banner.set_title(&err);
            widgets.banner.set_revealed(true);
        }
        None => widgets.banner.set_revealed(false),
    }

    clear_box(&widgets.updates_list);
    if updates.is_empty() {
        widgets
            .updates_list
            .append(&gtk::Label::new(Some("No recent friend updates.")));
    }
    for card in &updates {
        widgets.updates_list.append(&update_card(card, state, widgets));
    }

    clear_listbox(&widgets.following_list);
    for friend in &following {
        widgets.following_list.append(&friend_row(friend, state));
    }

    clear_listbox(&widgets.followers_list);
    for friend in &followers {
        widgets.followers_list.append(&friend_row(friend, state));
    }
}

fn scrolled_listbox() -> (gtk::ScrolledWindow, gtk::ListBox) {
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    list.add_css_class("boxed-list");
    list.set_margin_top(8);
    list.set_margin_bottom(8);
    list.set_margin_start(8);
    list.set_margin_end(8);
    let scrolled = gtk::ScrolledWindow::builder().child(&list).vexpand(true).build();
    (scrolled, list)
}

fn build_window(app: &adw::Application, controller: Controller) -> adw::ApplicationWindow {
    let state = Rc::new(RefCell::new(AppState {
        controller,
        avatar_cache: HashMap::new(),
    }));

    let updates_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let updates_scroller = gtk::ScrolledWindow::builder().child(&updates_list).vexpand(true).build();
    let (following_scroller, following_list) = scrolled_listbox();
    let (followers_scroller, followers_list) = scrolled_listbox();

    let stack = adw::ViewStack::new();
    stack.add_titled(&updates_scroller, Some("updates"), "Updates");
    stack.add_titled(&following_scroller, Some("following"), "Following");
    stack.add_titled(&followers_scroller, Some("followers"), "Followers");

    let switcher_title = adw::ViewSwitcherTitle::new();
    switcher_title.set_title("LuoDingo");
    switcher_title.set_stack(Some(&stack));

    let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh_button.set_tooltip_text(Some("Refresh"));

    let celebrate_all_button = gtk::Button::with_label("\u{1F389} Celebrate All");

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&switcher_title));
    header.pack_end(&refresh_button);
    header.pack_end(&celebrate_all_button);

    let banner = adw::Banner::new("");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&banner);
    content.append(&stack);

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&content));

    let widgets = Widgets {
        banner,
        celebrate_all_button,
        updates_list,
        following_list,
        followers_list,
    };

    render(&state, &widgets);

    refresh_button.connect_clicked({
        let state = state.clone();
        let widgets = widgets.clone();
        move |_| {
            info!("Refresh clicked");
            state.borrow_mut().controller.refresh();
            render(&state, &widgets);
        }
    });

    widgets.celebrate_all_button.connect_clicked({
        let state = state.clone();
        let widgets = widgets.clone();
        move |_| {
            let celebratable = state.borrow().controller.celebratable_count();
            info!("Celebrate All clicked: {celebratable} eligible");
            state.borrow_mut().controller.celebrate_all();
            render(&state, &widgets);
        }
    });

    adw::ApplicationWindow::builder()
        .application(app)
        .title("LuoDingo")
        .default_width(760)
        .default_height(640)
        .content(&toolbar_view)
        .build()
}

fn main() -> gtk::glib::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("starting luodingo-adwaita");

    let controller = match Controller::new() {
        Ok(c) => c,
        Err(e) => {
            log::error!("startup failed: {e}");
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let app = adw::Application::builder()
        .application_id("io.github.kjwenger.LuoDingo")
        .build();

    let controller = RefCell::new(Some(controller));
    app.connect_activate(move |app| {
        debug!("activate: building window");
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
        load_css();
        let controller = controller.borrow_mut().take().expect("activate called once");
        build_window(app, controller).present();
    });

    app.run()
}
