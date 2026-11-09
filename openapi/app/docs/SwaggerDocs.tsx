"use client";

import dynamic from "next/dynamic";
import "swagger-ui-react/swagger-ui.css";

// swagger-ui-react reaches for `window` at import time, so it can only run in the
// browser -- ssr: false keeps Next from trying to render it on the server.
const SwaggerUI = dynamic(() => import("swagger-ui-react"), { ssr: false });

export default function SwaggerDocs() {
  return <SwaggerUI url="/duolingo.yaml" />;
}
