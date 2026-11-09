# LuoDingo (Rust)

A small Rust client and desktop GUI for the unofficial Duolingo API. See the [top-level README](../README.md) for what LuoDingo is and why the API is "unofficial" — this document covers just the Rust implementation.

## What it does

- Authenticates using a JWT extracted from a logged-in browser session (see **Credentials** below).
- Fetches the friends activity feed (league promotions, kudos, follows — what's actually "recently achieved"), plus the lists of people you follow (`following`) and who follow you (`followers`), including each person's weekly/monthly/total XP and a recent-activity flag.
- Shows that data in a native desktop window, with an Updates tab (the achievement feed, styled as cards — avatar, name, timestamp, a league/milestone badge, and a working **Celebrate** button) plus Following/Followers tables, and a manual Refresh button — via **two independent GUI frontends** that share the same data/business logic (see **Layout**).
- Celebrate actually reacts on your real account: it calls Duolingo's `POST /card/reaction` with `reaction: "CONGRATS"` (reverse-engineered and verified live — see `core::api::react`), the same thing the real web app's "FEIERN" button does.

## Layout

This is a Cargo **workspace** with three members:

- **`core/`** (`luodingo-core`) — UI-toolkit-agnostic: the Duolingo API client (`src/api.rs`: JWT handling, user-id decoding, HTTP calls including `react`/`fetch_bytes`), the `Controller` that owns fetched state and refreshes/celebrates (`src/controller.rs`), and pure formatting helpers — timestamp/reaction/comment text, avatar URL normalization, and `format::badge()` which turns a card's `kudosIcon` string (e.g. `kudos_feed_leaderboard_rank_1`) into a glyph + color (gold/silver/bronze medal, league-promotion arrow, etc.) both frontends draw identically. Neither `egui` nor `gtk`/`libadwaita` appear anywhere in this crate.
- **`egui-app/`** (`luodingo-egui`) — a desktop window built with [`egui`](https://github.com/emilk/egui)/`eframe`. Pure-Rust rendering, no system GUI libraries required. Avatars load asynchronously via egui's built-in image loaders (`egui_extras::install_image_loaders`); the badge is hand-drawn with `Painter::circle_filled`/`text`.
- **`adwaita-app/`** (`luodingo-adwaita`) — the same data in a native GNOME-styled window, built with `gtk4-rs`/`libadwaita` (`adw::ViewStack` tabs, custom card widgets for Updates, `adw::ActionRow`+`adw::Avatar` for Following/Followers). Requires GTK4 + libadwaita installed on the system (not pure-Rust — see **Requirements**). Forces dark mode via `adw::StyleManager`; avatars are fetched with `core::fetch_bytes` and decoded with `gdk::Texture::from_bytes` (blocking, cached in-memory by URL so repeat refreshes don't re-download unchanged ones).

Both frontends construct a `luodingo_core::Controller` and just render its fields (`updates`, `following`, `followers`, `error`) — adding a third frontend means writing a new thin `src/main.rs`, not touching `core/` at all.

## Requirements

- Rust (install via [rustup](https://rustup.rs) if you don't have it: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).
- A graphical display (X11 or Wayland) to show the window — these are native GUI apps, not headless/terminal tools.
- For `adwaita-app` only: GTK4 and libadwaita development packages (on Debian/Ubuntu: `sudo apt install pkg-config libgtk-4-dev libadwaita-1-dev`). `egui-app` needs none of this — it's pure Rust.

## Credentials

The app reads a single variable, `DUOLINGO_JWT_TOKEN`, from a `.env` file in this directory.

1. Copy the example file: `cp .env.example .env`
2. Get your JWT: log into [duolingo.com](https://www.duolingo.com) in a browser, open DevTools (F12) → **Storage**/**Application** → **Cookies** → `https://www.duolingo.com`, and copy the value of the `jwt_token` cookie.
3. Paste it into `.env` as the value of `DUOLINGO_JWT_TOKEN`.

Username/password login against Duolingo's `/login` endpoint does **not** work here — it's bot-gated and silently returns the web app's HTML shell instead of a token. The JWT-from-browser approach above is the one that actually authenticates.

`.env` is listed in `.gitignore` at the repo root — never commit it. Treat the JWT like a password.

## Build & run

This is a workspace with two binaries, so pick one with `-p`:

```bash
cargo build --workspace        # compile everything
cargo run -p luodingo-egui     # launch the egui frontend
cargo run -p luodingo-adwaita  # launch the GNOME/libadwaita frontend
```

Run these from this directory (`rust/`) so `.env` is found — `Controller::new()` loads it via `dotenvy`, which searches the current directory upward. On first run, dependencies are fetched and compiled, which can take a minute or two (longer for `adwaita-app`, since GTK4/libadwaita are larger C libraries being linked against). Subsequent runs are fast.

## Known limitations

- The activity feed (`GET /2023-05-23/friends/users/{id}/feed/v2`) only surfaces what Duolingo's own kudos system tracks — league promotions, streak/league milestones, follows, etc. It doesn't include every possible action (e.g. ordinary lesson completions aren't cards unless they trigger a kudos-worthy milestone).
- Some users have no display name set (`name` is `null` in the API response); the app falls back to their username in the Following/Followers tables, and to "Someone" in the Updates feed.
- Refresh is manual (no auto-polling) and runs on the UI thread, so the window will briefly freeze while a refresh is in flight.
- Each update shows reaction counts (🎉/❤️/👍/etc., from `reactionCounts`) when present. Comment counts (`commentPreview`) are shown too, but that field was `null` on every card observed so far — its non-null shape is unconfirmed, so the comment indicator falls back to a bare 💬 icon (no number) if a real comment payload doesn't match the field names we guessed (`totalComments`/`commentCount`/`count`).
- `adwaita-app` uses `adw::ViewSwitcherTitle`, which libadwaita deprecated in 1.4 in favor of a newer adaptive-layout pattern (`adw::ViewSwitcher` + `adw::Breakpoint`). It still works (just a deprecation warning at build time) and was the simplest widget for a first pass; worth revisiting if this frontend gets more attention.
- The card design (avatar, badge, name/time, body, Celebrate button) is modeled on a screenshot of the real `duolingo.com/friend-updates` page, not pixel-matched: we never found a confirmed URL for Duolingo's actual badge artwork (`kudosIcon` is just an identifier string, e.g. `kudos_feed_leaderboard_rank_1`), so badges are drawn as plain colored circles with a glyph instead of real medal images.
- `adwaita-app` fetches each avatar image synchronously (blocking) the first time it's seen, so the window can pause noticeably on first load or refresh if you follow a lot of people; `egui-app` doesn't have this problem since egui's image loader fetches asynchronously in the background.
- The Celebrate button always sends `"CONGRATS"` — the real app likely has distinct reactions for different buttons/emoji (we saw `like`, `haha`, `cheer`, `high_five`, `love`, `support` as existing reaction *types* in feed data), but we only reverse-engineered and verified the one the visible "Celebrate" button uses.
