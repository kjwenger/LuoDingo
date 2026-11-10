"use client";

import dynamic from "next/dynamic";
import "swagger-ui-react/swagger-ui.css";

// swagger-ui-react reaches for `window` at import time, so it can only run in the
// browser -- ssr: false keeps Next from trying to render it on the server.
const SwaggerUI = dynamic(() => import("swagger-ui-react"), { ssr: false });

// Public-folder assets aren't auto-prefixed by Next.js's basePath the way next/link,
// next/navigation, and next/image are -- has to be done by hand. NEXT_PUBLIC_BASE_PATH
// mirrors next.config.ts's NEXT_BASE_PATH build arg so the client bundle has it inlined.
const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

export default function SwaggerDocs() {
  // swagger-ui-react isn't dark-mode aware -- it sets its own (dark) text colors but
  // never a background, expecting a plain white page. Our global dark-mode body
  // background (app/globals.css, prefers-color-scheme: dark) left that text essentially
  // invisible: dark-on-near-black. Force a light background here regardless of the
  // system/browser color scheme so the page stays legible either way.
  return (
    <div style={{ background: "#fff", colorScheme: "light", minHeight: "100vh" }}>
      <SwaggerUI url={`${basePath}/duolingo.yaml`} />
    </div>
  );
}
