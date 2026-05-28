import { describe, expect, it } from "vitest";

import {
  applyThemePreference,
  parseThemePreference,
  sanitizeCustomThemeTokens,
} from "./theme";

describe("theme helpers", () => {
  it("parses known theme preferences", () => {
    expect(parseThemePreference("light")).toBe("light");
    expect(parseThemePreference("dark")).toBe("dark");
    expect(parseThemePreference("system")).toBe("system");
  });

  it("falls back to system for unknown preferences", () => {
    expect(parseThemePreference("sepia")).toBe("system");
    expect(parseThemePreference(null)).toBe("system");
  });

  it("keeps only allowlisted custom theme variables", () => {
    expect(
      sanitizeCustomThemeTokens({
        "--color-bg-app": " #ffffff ",
        "--space-8": "999px",
        color: "red",
      }),
    ).toEqual({ "--color-bg-app": "#ffffff" });
  });

  it("drops unsafe custom token values", () => {
    expect(
      sanitizeCustomThemeTokens({
        "--color-bg-app": "red; color: blue",
        "--color-danger": "url(https://example.invalid/pixel)",
        "--color-warning": "@import url(https://example.invalid/theme.css)",
        "--color-text": "var(--color-accent)",
      }),
    ).toEqual({ "--color-text": "var(--color-accent)" });
  });

  it("allows constrained color syntaxes for custom token values", () => {
    expect(
      sanitizeCustomThemeTokens({
        "--color-accent": "rgb(35 104 154 / 92%)",
        "--color-accent-muted": "color-mix(in srgb, var(--color-accent) 34%, transparent)",
        "--color-focus": "var(--color-accent-strong)",
        "--color-surface": "calc(1px + 2px)",
      }),
    ).toEqual({
      "--color-accent": "rgb(35 104 154 / 92%)",
      "--color-accent-muted": "color-mix(in srgb, var(--color-accent) 34%, transparent)",
      "--color-focus": "var(--color-accent-strong)",
    });
  });

  it("applies explicit and system theme preferences", () => {
    const root = { dataset: {} } as HTMLElement;

    applyThemePreference("dark", root);
    expect(root.dataset.theme).toBe("dark");

    applyThemePreference("system", root);
    expect(root.dataset.theme).toBe("system");
  });
});
