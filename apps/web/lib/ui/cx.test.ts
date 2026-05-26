import { describe, expect, it } from "vitest";

import { cx } from "./cx";

describe("cx", () => {
  it("joins truthy class names", () => {
    expect(cx("button", "buttonActive")).toBe("button buttonActive");
  });

  it("skips falsey class names without extra whitespace", () => {
    expect(cx("button", false, null, undefined, "large")).toBe("button large");
  });
});
