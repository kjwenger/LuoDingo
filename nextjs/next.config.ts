import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Minimal, self-contained production output (.next/standalone) for the Docker image --
  // see Dockerfile.
  output: "standalone",

  // Set at build time only when this build is deployed behind a reverse proxy under a
  // URL prefix (e.g. https://host/luodingo/...) -- see Dockerfile's NEXT_BASE_PATH build
  // arg. Empty by default so direct-port/root-mounted deployments (the documented
  // default) are unaffected. next/link and next/navigation pick this up automatically;
  // anything that isn't one of those (plain <a>, a client fetch of a public/ asset) has
  // to be prefixed by hand -- see app/page.tsx and app/docs/SwaggerDocs.tsx.
  basePath: process.env.NEXT_BASE_PATH || "",

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
