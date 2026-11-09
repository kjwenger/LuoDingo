# LuoDingo (OpenAPI / Next.js)

A reverse-engineered **OpenAPI 3.0** description of the unofficial Duolingo endpoints LuoDingo uses, served via a [Swagger UI](https://swagger.io/tools/swagger-ui/) page built with [Next.js](https://nextjs.org). See the [top-level README](../README.md) for what LuoDingo is and why the API is "unofficial".

This exists for two reasons:

1. **Documentation** — a browsable, testable reference for the endpoints the [Rust implementation](../rust/README.md) already uses, instead of that knowledge living only in `rust/core/src/api.rs` and this repo's commit history.
2. **A second frontend, eventually** — once the spec is solid, the plan is to build a web UI here with the same functionality as the Rust `egui`/`adwaita` apps (friend updates feed, Celebrate, etc.), as a second implementation sharing nothing with the Rust one except the API knowledge encoded in the spec.

Today it's just the spec + docs page — no API routes or dashboard yet.

## What's here

- **`public/duolingo.yaml`** — the OpenAPI spec itself. Covers the five endpoints LuoDingo's Rust client calls: profile, following, followers, the friends activity feed, and reacting to a feed card ("Celebrate"). Served statically at `/duolingo.yaml` (anything under `public/` is served at the site root by Next.js).
- **`app/docs/`** — the Swagger UI page at `/docs`, pointed at that spec file.
- **`app/page.tsx`** — a minimal landing page linking to `/docs`.

## Why the spec looks the way it does

Nothing here is documented by Duolingo — there is no official API. Every path, field, and quirk in `duolingo.yaml` was found by reading the Rust client's source (where it was reverse-engineered from the real web client's network traffic) and in several cases verified again directly against the live API while writing this spec. Notable things worth knowing before trusting it blindly:

- `/card/reaction` has no date-version prefix, unlike every other path (`/2017-06-30/...`, `/2023-05-23/...`).
- `reaction: "CONGRATS"` is the only reaction value actually confirmed live (verified: it took a card from no reaction to `reactionCounts: {"congrats": 1}`). Other values almost certainly exist (`reactionCounts` keys seen in real feed data include `like`, `haha`, `cheer`, `high_five`, `love`, `support`) but aren't confirmed.
- `kudosIcon` (on feed cards) identifies which badge the real app shows (e.g. `kudos_feed_leaderboard_rank_1`) — it's an identifier string, not an image URL. No confirmed URL for the actual badge artwork was ever found.
- `picture` (avatar URLs, on both friends and feed cards) is sometimes protocol-relative and **always** needs a trailing size segment (e.g. `/medium`) appended to actually load — omitting it returns `400` or `403` depending on which avatar host it is. This isn't modeled in the spec itself (OpenAPI doesn't have a clean way to express "append this to use the value"), just noted in the field description.
- `commentPreview` is typed as an open/unknown object because every card observed during development had it as `null` — its real shape when populated is unconfirmed.

If you change how the Rust client calls these endpoints, update this spec to match (and vice versa) — right now nothing keeps them in sync automatically.

## Requirements

- Node.js (developed against v22) and npm.

## Build & run

```bash
# install dependencies
npm install

# dev server with hot reload, at http://localhost:3000
npm run dev

# production build + start
npm run build
npm run start
```

Then open `/docs` for the Swagger UI, or `/duolingo.yaml` to fetch the raw spec.

## Validating the spec

```bash
npx @redocly/cli lint public/duolingo.yaml
```

Run this after editing `duolingo.yaml` — it catches structural mistakes (bad `$ref`s, schema issues like a `nullable` field with no `type`, etc.) that are easy to introduce by hand and that Swagger UI won't necessarily complain about loudly.
