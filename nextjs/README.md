# LuoDingo (Next.js)

[![Lint](https://github.com/kjwenger/LuoDingo/actions/workflows/nextjs-lint.yml/badge.svg)](https://github.com/kjwenger/LuoDingo/actions/workflows/nextjs-lint.yml)
[![Last commit affecting nextjs/](https://img.shields.io/github/last-commit/kjwenger/LuoDingo?path=nextjs&label=last%20commit%20(nextjs%2F))](https://github.com/kjwenger/LuoDingo/commits/main/nextjs)
[![Next.js](https://img.shields.io/badge/Next.js-000000?logo=nextdotjs&logoColor=white)](https://nextjs.org)
[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org)
[![Docker](https://img.shields.io/badge/Docker-2496ED?logo=docker&logoColor=white)](Dockerfile)

A reverse-engineered **OpenAPI 3.0** description of the unofficial Duolingo endpoints LuoDingo uses, served via a [Swagger UI](https://swagger.io/tools/swagger-ui/) page, **plus a web UI with equivalent functionality to the Rust `egui`/`adwaita` desktop apps** (friend updates feed, Celebrate/Celebrate All), built with [Next.js](https://nextjs.org). See the [top-level README](../README.md) for what LuoDingo is and why the API is "unofficial". Shares nothing with the Rust implementation except the API knowledge encoded in the spec — independently reimplemented.

## What's here

- **`public/duolingo.yaml`** — the OpenAPI spec. Served statically at `/duolingo.yaml`; browsable at `/docs` via Swagger UI (`app/docs/`).
- **`lib/duolingo/`** — server-only Duolingo client (`client.ts`, `import "server-only"`), plain TS types (`types.ts`), and pure formatting helpers (`format.ts`) mirroring `rust/core/src/{api,format}.rs` field-for-field and quirk-for-quirk (avatar URL `/medium` suffix, `CONGRATS` reaction value, `kudosIcon`-not-`triggerType` for badges).
- **`app/feed/`** — the friend-updates UI at `/feed`. `page.tsx` is a Server Component (`export const dynamic = "force-dynamic"`) that fetches feed/following/followers in parallel and keeps partial results on partial failure (mirrors `controller.rs::refresh()`); `actions.ts` has the two Server Actions (`celebrateAction`, `celebrateAllAction`) that call Duolingo then `revalidatePath("/feed")`; `FeedView.tsx` is the Client Component (tabs, cards, buttons).
- **`app/page.tsx`** — landing page linking to `/feed` and `/docs`.
- **`Dockerfile`** — multi-stage build producing a minimal standalone production image (see **Docker** below).

## Why `cacheComponents` is off

`next.config.ts` had `cacheComponents: true` (Next 16's default from `create-next-app`). Under it, a Server Component reading live/uncached data **must** sit behind a `<Suspense>` boundary — a build error otherwise, not a warning. That machinery exists for multi-tenant apps caching per-session data; we have one account/JWT and want every read live with zero caching. Turned it off and used the classic model instead (`force-dynamic`, Server Actions, uncached `fetch` by default) — simpler and a better fit, with no loss since `/docs` doesn't read dynamic data either way.

## Credentials

Copy `.env.local.example` to `.env.local` and fill in `DUOLINGO_JWT_TOKEN` (same value as `rust/.env`'s — see the top-level README's **Credentials** section for how to extract one). It's read server-only (`lib/duolingo/client.ts` imports `"server-only"`) and never reaches the browser. `.env*` is already gitignored. No `npm run build`/CI needs it set — `/feed` is `force-dynamic`, so no live fetch is attempted at build time.

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

Then open `/docs` for the Swagger UI, `/feed` for the friend updates UI, or `/duolingo.yaml` to fetch the raw spec.

## Docker

`Dockerfile` is a standard multi-stage build on Next.js's `output: "standalone"` (set in `next.config.ts`) — the final image only carries the trace-selected `node_modules` subset, not the full dependency tree (~236MB, `node:22-alpine` base).

```bash
docker build -t luodingo-nextjs .

# DUOLINGO_JWT_TOKEN is read at runtime (process.env, server-only), never baked into the
# image -- pass it at `docker run`, the same way you'd promote one image through
# multiple environments with different secrets.
docker run -p 3000:3000 -e DUOLINGO_JWT_TOKEN="$(grep -oP '(?<=DUOLINGO_JWT_TOKEN=).*' .env.local)" luodingo-nextjs
```

No secret is needed to *build* the image — same reasoning as the CI build step (`/feed` is `force-dynamic`, so no live fetch happens at build time). CI builds the image on every push touching `nextjs/**` to catch Dockerfile bitrot, but doesn't push it anywhere.

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
