# LuoDingo (Rust)

A small Rust client and desktop GUI for the unofficial Duolingo API. See the [top-level README](../README.md) for what LuoDingo is and why the API is "unofficial" — this document covers just the Rust implementation.

## What it does

- Authenticates using a JWT extracted from a logged-in browser session (see **Credentials** below).
- Fetches the friends activity feed (league promotions, kudos, follows — what's actually "recently achieved"), plus the lists of people you follow (`following`) and who follow you (`followers`), including each person's weekly/monthly/total XP and a recent-activity flag.
- Shows that data in a native desktop window, with an Updates tab (the achievement feed) plus Following/Followers tables, and a manual Refresh button — via **two independent GUI frontends** that share the same data/business logic (see **Layout**).

## Layout

This is a Cargo **workspace** with three members:

- **`core/`** (`luodingo-core`) — UI-toolkit-agnostic: the Duolingo API client (`src/api.rs`: JWT handling, user-id decoding, HTTP calls), the `Controller` that owns fetched state and refreshes it (`src/controller.rs`), and pure formatting helpers like timestamp/reaction/comment rendering (`src/format.rs`). Neither `egui` nor `gtk`/`libadwaita` appear anywhere in this crate — both frontends below depend on it, not the other way around.
- **`egui-app/`** (`luodingo-egui`) — a desktop window built with [`egui`](https://github.com/emilk/egui)/`eframe`. Pure-Rust rendering, no system GUI libraries required.
- **`adwaita-app/`** (`luodingo-adwaita`) — the same data in a native GNOME-styled window, built with `gtk4-rs`/`libadwaita` (`adw::ViewStack` tabs, `adw::ActionRow` list rows). Requires GTK4 + libadwaita installed on the system (not pure-Rust — see **Requirements**).

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

```sh
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
