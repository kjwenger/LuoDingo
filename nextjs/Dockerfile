# syntax=docker/dockerfile:1
#
# Multi-stage build using Next.js's "standalone" output (next.config.ts: output:
# "standalone") so the final image only needs the trace-selected node_modules subset,
# not the full dependency tree. See https://nextjs.org/docs/app/guides/self-hosting.

FROM node:22-alpine AS base

# ---- deps: install dependencies only (cached separately from source changes) ----
FROM base AS deps
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci

# ---- builder: build the app ----
FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
# No DUOLINGO_JWT_TOKEN needed here: app/feed/page.tsx is force-dynamic, so Next.js never
# attempts the live Duolingo fetch at build time (verified locally with the var unset).
#
# Baked in at build time for reverse-proxy-under-a-prefix deployments (e.g. JoNAS's
# /luodingo/ path) -- empty by default so a plain `docker build` stays root-mounted, the
# documented default. See next.config.ts. NEXT_PUBLIC_* vars are inlined into the client
# bundle by `next build` itself reading process.env, so these need to be real build-stage
# ENV vars, not just Docker ARGs.
ARG NEXT_BASE_PATH=
ARG NEXT_PUBLIC_BASE_PATH=
ENV NEXT_BASE_PATH=$NEXT_BASE_PATH
ENV NEXT_PUBLIC_BASE_PATH=$NEXT_PUBLIC_BASE_PATH
RUN npm run build

# ---- runner: minimal production image ----
FROM base AS runner
WORKDIR /app
ENV NODE_ENV=production

RUN addgroup --system --gid 1001 nodejs \
  && adduser --system --uid 1001 nextjs

COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs

EXPOSE 3000
ENV PORT=3000
ENV HOSTNAME=0.0.0.0

# DUOLINGO_JWT_TOKEN is read at runtime (process.env, server-only -- see
# lib/duolingo/client.ts) -- pass it with `docker run -e DUOLINGO_JWT_TOKEN=...` or
# --env-file, never bake it into the image.
CMD ["node", "server.js"]
