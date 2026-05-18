import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

function readServerPage(): string {
  return readFileSync(join(process.cwd(), "app", "servers", "[serverId]", "page.tsx"), "utf8");
}

describe("server actions accessibility", () => {
  it("uses disclosure semantics instead of ARIA menu roles", () => {
    const source = readServerPage();

    expect(source).toContain('aria-controls="server-actions-panel"');
    expect(source).toContain('id="server-actions-panel"');
    expect(source).not.toContain('role="menu"');
    expect(source).not.toContain('role="menuitem"');
  });

  it("returns focus to the disclosure button after an action closes", () => {
    const source = readServerPage();

    expect(source).toContain("const serverMenuButtonRef = useRef<HTMLButtonElement | null>(null);");
    expect(source).toContain("ref={serverMenuButtonRef}");
    expect(source).toContain("serverMenuButtonRef.current?.focus();");
  });
});
