import { describe, expect, it } from "vitest";

import { initials } from "./initials";

describe("initials", () => {
  it("uses the first two name segments", () => {
    expect(initials("Ada Lovelace")).toBe("AL");
    expect(initials("  grace  brewster hopper ")).toBe("GB");
  });

  it("falls back for blank names", () => {
    expect(initials("   ")).toBe("?");
  });
});
