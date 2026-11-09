# LuoDingo (OpenAPI / Next.js)

A reverse-engineered **OpenAPI 3.0** description of the unofficial Duolingo endpoints LuoDingo uses, served via a [Swagger UI](https://swagger.io/tools/swagger-ui/) page built with [Next.js](https://nextjs.org). See the [top-level README](../README.md) for what LuoDingo is and why the API is "unofficial".

This exists for two reasons:

1. **Documentation** — a browsable, testable reference for the endpoints the [Rust implementation](../rust/README.md) already uses, instead of that knowledge living only in `rust/core/src/api.rs` and this repo's commit history.
2. **A second frontend, eventually** — once the spec is solid, the plan is to build a web UI here with the same functionality as the Rust `egui`/`adwaita` apps (friend updates feed, Celebrate, etc.), as a second implementation sharing nothing with the Rust one except the API knowledge encoded in the spec.

Today it's just the spec + docs page — no API routes or dashboard yet.

## What's here

- **`public/duolingo.yaml`** — the OpenAPI spec itself. Covers the five endpoints LuoDingo's Rust client calls (profile, following, followers, the friends activity feed, reacting to a feed card), plus a few more found cheap (read-only) to verify live: the full profile shape, richer feed-card fields, the feed response's `kudosConfig`/`newsConfig`/`sentenceConfig` (which reactions are valid per card category), and a newer paginated following/followers endpoint. One write endpoint (`follow`) is documented from reading a third-party client's source only, never called — see below. Served statically at `/duolingo.yaml` (anything under `public/` is served at the site root by Next.js).
- **`app/docs/`** — the Swagger UI page at `/docs`, pointed at that spec file. Forces a light background on itself (`SwaggerDocs.tsx`) regardless of system/browser dark-mode preference — `swagger-ui-react` sets its own dark text colors but no background of its own, so without this it rendered as dark-on-near-black (our global dark-mode CSS) and looked completely blank under a dark color scheme. Found with `scripts/check-page.mjs` (see **Debugging**), since it wasn't reproducible by just glancing at `next build` output or testing in a light-mode browser.
- **`app/page.tsx`** — a minimal landing page linking to `/docs`.

## Why the spec looks the way it does

Nothing here is documented by Duolingo — there is no official API. Every path, field, and quirk in `duolingo.yaml` was found by reading the Rust client's source (where it was reverse-engineered from the real web client's network traffic) and in several cases verified again directly against the live API while writing this spec. Notable things worth knowing before trusting it blindly:

- `/card/reaction` has no date-version prefix, unlike every other path (`/2017-06-30/...`, `/2023-05-23/...`).
- `reaction: "CONGRATS"` is the only reaction value actually confirmed live (verified: it took a card from no reaction to `reactionCounts: {"congrats": 1}`). Other values almost certainly exist (`reactionCounts` keys seen in real feed data include `like`, `haha`, `cheer`, `high_five`, `love`, `support`) but aren't confirmed.
- `kudosIcon` (on feed cards) identifies which badge the real app shows (e.g. `kudos_feed_leaderboard_rank_1`) — it's an identifier string, not an image URL. No confirmed URL for the actual badge artwork was ever found.
- `picture` (avatar URLs, on both friends and feed cards) is sometimes protocol-relative and **always** needs a trailing size segment (e.g. `/medium`) appended to actually load — omitting it returns `400` or `403` depending on which avatar host it is. This isn't modeled in the spec itself (OpenAPI doesn't have a clean way to express "append this to use the value"), just noted in the field description.
- `commentPreview` is typed as an open/unknown object because every card observed during development had it as `null` — its real shape when populated is unconfirmed.
- The feed response's `kudosConfig`/`newsConfig`/`sentenceConfig` list every valid `reactionType` per card category (`congrats`/`high_five`/`support`/`cheer`/`love` for kudos cards; `haha`/`like`/`love` for news/sentence cards) — confirmed live, straight from Duolingo's own response, not guessed. Only `congrats` (as `CONGRATS`) has actually been POST-verified against `/card/reaction`, though; the other four kudos reactions and all three news/sentence ones are "found", not individually confirmed to work.
- `triggerType` turned out to be a red herring for picking a card's badge — both "Finished #1" and "Finished top 3" cards had `triggerType: "top_three"`. `kudosIcon` is the field that actually distinguishes them (`kudos_feed_leaderboard_rank_1` vs `_rank_2`); the Rust client already does this right, the spec just documents the pitfall for choosing `triggerType` instead.
- There's a second, newer `following`/`followers` pair (`/2023-05-23/friends/users/{id}/following` and `/followers`) alongside the one actually used. It's genuinely paginated (a `cursor`, unlike the older endpoint) and returns richer per-user data (mutual-follow status, active status, a leaderboard-style score) — but drops `weeklyXp`/`monthlyXp`, which is the only reason the Rust client uses the older one instead. Pagination itself was never exercised: the test account's lists both fit on one page, so `cursor` always came back `null`.
- The `follow` endpoint (`POST /2023-05-23/friends/users/{id}/follow/{id}`) is the one thing in the spec **not** verified live — it changes a real relationship visible to another account, which isn't something to test just to document an endpoint. It's there because a third-party reverse-engineered client's source mentions it, flagged clearly as unconfirmed.

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

## Debugging

`scripts/check-page.mjs` loads a URL in a real, system-installed Chrome (via Playwright, `channel: "chrome"` — no bundled browser download) and reports console messages, page errors, failed requests, the rendered text, and optionally a screenshot. Useful for exactly the kind of bug that doesn't show up in build output or server logs — e.g. the dark-mode/Swagger UI contrast issue above only showed up when actually rendering the page with a dark color scheme, not from `npm run build` or `npm run lint` succeeding:

```bash
# start a server first (separate terminal)
npm run dev -- -p 3100

# then, in another terminal
npm run check-page -- http://localhost:3100/docs --dark --out /tmp/docs.png
```

`--dark` emulates a dark browser/OS color-scheme preference; omit it to use the default (light). `--out <path>` saves a screenshot.
