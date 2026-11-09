# LuoDingo (Rust)

A small Rust client and desktop GUI for the unofficial Duolingo API. See the [top-level README](../README.md) for what LuoDingo is and why the API is "unofficial" — this document covers just the Rust implementation.

## What it does

- Authenticates using a JWT extracted from a logged-in browser session (see **Credentials** below).
- Fetches the friends activity feed (league promotions, kudos, follows — what's actually "recently achieved"), plus the lists of people you follow (`following`) and who follow you (`followers`), including each person's weekly/monthly/total XP and a recent-activity flag.
- Shows that data in a native desktop window (built with [`egui`](https://github.com/emilk/egui)/`eframe`), with an Updates tab (the achievement feed) plus Following/Followers tables, and a manual Refresh button.

## Layout

- `src/api.rs` — the Duolingo API client: JWT handling, user-id decoding, and the HTTP calls (`profile`, `following`, `followers`, `feed`).
- `src/main.rs` — the `egui` application: window state, tabs, the updates list, and the friends table.

## Requirements

- Rust (install via [rustup](https://rustup.rs) if you don't have it: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).
- A graphical display (X11 or Wayland) to show the window — this is a native GUI app, not a headless/terminal tool.

## Credentials

The app reads a single variable, `DUOLINGO_JWT_TOKEN`, from a `.env` file in this directory.

1. Copy the example file: `cp .env.example .env`
2. Get your JWT: log into [duolingo.com](https://www.duolingo.com) in a browser, open DevTools (F12) → **Storage**/**Application** → **Cookies** → `https://www.duolingo.com`, and copy the value of the `jwt_token` cookie.
3. Paste it into `.env` as the value of `DUOLINGO_JWT_TOKEN`.

Username/password login against Duolingo's `/login` endpoint does **not** work here — it's bot-gated and silently returns the web app's HTML shell instead of a token. The JWT-from-browser approach above is the one that actually authenticates.

`.env` is listed in `.gitignore` at the repo root — never commit it. Treat the JWT like a password.

## Build & run

```sh
cargo build       # compile
cargo run         # build (if needed) and launch the GUI
```

On first run, dependencies are fetched and compiled, which can take a minute or two. Subsequent runs are fast.

## Known limitations

- The activity feed (`GET /2023-05-23/friends/users/{id}/feed/v2`) only surfaces what Duolingo's own kudos system tracks — league promotions, streak/league milestones, follows, etc. It doesn't include every possible action (e.g. ordinary lesson completions aren't cards unless they trigger a kudos-worthy milestone).
- Some users have no display name set (`name` is `null` in the API response); the app falls back to their username in the Following/Followers tables, and to "Someone" in the Updates feed.
- Refresh is manual (no auto-polling) and runs on the UI thread, so the window will briefly freeze while a refresh is in flight.
