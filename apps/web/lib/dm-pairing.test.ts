import { describe, expect, it } from "vitest";

import { buildDmPairingLink, buildDmPairingManualCode, parseDmPairingInput } from "./dm-pairing";

describe("dm pairing helpers", () => {
  it("builds a dm pairing deep link", () => {
    expect(buildDmPairingLink("pairing-envelope-123")).toBe(
      "hexrelay://dm-pairing/pairing-envelope-123",
    );
  });

  it("builds a whitespace-grouped manual code", () => {
    expect(buildDmPairingManualCode("abcdefghijklmnopq")).toBe("DM1 abcdefgh ijklmnop q");
  });

  it("parses a raw pairing envelope", () => {
    expect(parseDmPairingInput("pairing-envelope-123")).toBe("pairing-envelope-123");
  });

  it("parses a dm pairing deep link", () => {
    expect(parseDmPairingInput("hexrelay://dm-pairing/pairing-envelope-123")).toBe(
      "pairing-envelope-123",
    );
  });

  it("parses a DM1 manual code", () => {
    expect(parseDmPairingInput("DM1 pairing-\nenvelope_ 123")).toBe("pairing-envelope_123");
  });

  it("parses a deep link with query or fragment suffixes", () => {
    expect(parseDmPairingInput("hexrelay://dm-pairing/pairing-envelope-123?via=qr#scan")).toBe(
      "pairing-envelope-123",
    );
  });

  it("returns null for empty input", () => {
    expect(parseDmPairingInput("   ")).toBeNull();
  });
});
