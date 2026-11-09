import type { Metadata } from "next";
import SwaggerDocs from "./SwaggerDocs";

export const metadata: Metadata = {
  title: "Duolingo API (Unofficial) — LuoDingo",
  description: "Reverse-engineered OpenAPI spec for the unofficial Duolingo endpoints LuoDingo uses.",
};

export default function DocsPage() {
  return <SwaggerDocs />;
}
