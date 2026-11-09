import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // cacheComponents (and partialPrefetching, which requires it) are off: this app has a
  // single account/JWT and wants every read to hit Duolingo live with zero caching, which
  // is what cacheComponents' mandatory Suspense-boundary-for-dynamic-data model is built to
  // avoid needing. The classic model (plain `export const dynamic = "force-dynamic"`,
  // uncached fetch by default) fits better and needs none of that machinery. See
  // nextjs/README.md.
  turbopack: {
    rules: {
      "*.css": {
        loaders: ["@tailwindcss/turbopack"],
        as: "*.css",
      },
    },
  },
};

export default nextConfig;
