import { notFound } from "next/navigation";

import { Demo } from "./demo";

export default function UiCatalogPage() {
  if (process.env.NODE_ENV === "production") {
    notFound();
  }

  return <Demo />;
}
