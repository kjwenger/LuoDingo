# LuoDingo

[![Rust](https://github.com/kjwenger/LuoDingo/actions/workflows/rust.yml/badge.svg)](https://github.com/kjwenger/LuoDingo/actions/workflows/rust.yml)
[![OpenAPI (Next.js) Lint](https://github.com/kjwenger/LuoDingo/actions/workflows/openapi-lint.yml/badge.svg)](https://github.com/kjwenger/LuoDingo/actions/workflows/openapi-lint.yml)
[![License: Unlicense](https://img.shields.io/github/license/kjwenger/LuoDingo)](UNLICENSE)
[![Last commit](https://img.shields.io/github/last-commit/kjwenger/LuoDingo)](https://github.com/kjwenger/LuoDingo/commits/main)
[![Open issues](https://img.shields.io/github/issues/kjwenger/LuoDingo)](https://github.com/kjwenger/LuoDingo/issues)
[![Top language](https://img.shields.io/github/languages/top/kjwenger/LuoDingo)](https://github.com/kjwenger/LuoDingo)

LuoDingo is a personal toolkit for working with your own [Duolingo](https://www.duolingo.com) account data through Duolingo's **unofficial** REST API. Duolingo doesn't publish a public API, but the endpoints its own web and mobile clients use can be called directly once you're authenticated as yourself.

Today that means pulling your profile stats and seeing XP/activity updates for the people you follow and the people who follow you (a lightweight stand-in for Duolingo's in-app "Friends" feed). The project isn't tied to any particular language or UI — the `rust/` folder holds the current implementation, but the ideas here (auth, endpoints, data shape) apply to any client you might build on top of them.

## Why "unofficial"?

There is no supported, documented Duolingo API for third-party apps. Everything here talks to the same endpoints Duolingo's own web client uses (`https://www.duolingo.com/2017-06-30/...`). That means:

- Endpoints and fields can change or disappear without notice.
- Duolingo's Terms of Service govern what you're allowed to do with your account and its data — this project is for personal, read-only use against your own account, not for scraping other users' data at scale or automating gameplay.
- Programmatic username/password login is bot-gated and does not work; see **Credentials** below for the approach that does.

## What's here

- **[`rust/`](rust/README.md)** — a Rust Cargo workspace: a UI-agnostic core (the API client and app state) plus two interchangeable desktop GUI frontends built on top of it — one with [`egui`](https://github.com/emilk/egui) (pure Rust, no system dependencies) and one with GTK4/libadwaita (native GNOME look and feel). See that folder's README for build/run instructions and implementation details.
- **[`openapi/`](openapi/README.md)** — a reverse-engineered OpenAPI spec for the same endpoints, served as a browsable Swagger UI page via Next.js. Currently just documentation; the plan is to eventually build a web UI here with equivalent functionality to the Rust desktop apps, as a second independent frontend.

## Credentials

LuoDingo authenticates using a **JWT token** copied out of a browser session, not your Duolingo password. Duolingo's `/login` endpoint no longer returns a usable token for non-browser clients, so the practical approach is:

1. Log into [duolingo.com](https://www.duolingo.com) in your browser as normal.
2. Open your browser's developer tools (F12), go to the **Storage**/**Application** tab, and find the `jwt_token` cookie under `https://www.duolingo.com`.
3. Copy its value into your local `.env` file (see `rust/.env.example` for the expected format — copy it to `rust/.env` and fill in `DUOLINGO_JWT_TOKEN`).

This JWT encodes your Duolingo user ID and, in practice, doesn't expire on its own — but it **will** stop working if you change your Duolingo password, at which point you'll need to extract a fresh one.

**Never commit `.env`.** It already is covered by `.gitignore` at the repo root, but double-check before pushing if you ever restructure the project. Treat the JWT like a password: anyone with it can act as you against the Duolingo API.
