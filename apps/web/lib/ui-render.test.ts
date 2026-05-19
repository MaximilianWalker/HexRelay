import { createElement, type AnchorHTMLAttributes, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, describe, expect, it, vi } from "vitest";

import ContactsPage from "../app/contacts/page";
import ContactMessagesPage from "../app/contacts/[contactId]/messages/page";
import HomePage from "../app/home/page";
import IdentityOnboardingPage from "../app/onboarding/identity/page";

const navigation = vi.hoisted(() => ({
  params: {} as Record<string, string>,
  pathname: "/home",
  push: vi.fn(),
  redirect: vi.fn((target: string) => {
    throw new Error(`redirect:${target}`);
  }),
}));

type LinkProps = Omit<AnchorHTMLAttributes<HTMLAnchorElement>, "href"> & {
  children: ReactNode;
  href: string;
};

vi.mock("next/link", () => ({
  default: function Link({ children, href, ...props }: LinkProps) {
    return createElement("a", { ...props, href: String(href) }, children);
  },
}));

vi.mock("next/navigation", () => ({
  redirect: navigation.redirect,
  useParams: () => navigation.params,
  usePathname: () => navigation.pathname,
  useRouter: () => ({
    push: navigation.push,
  }),
}));

vi.mock("@/lib/crypto", () => ({
  bytesToHex: (bytes: Uint8Array) => Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join(""),
  derivePublicKey: () => new Uint8Array(32),
  generateIdentityKeypair: () => ({
    privateKeyHex: "a".repeat(128),
    publicKeyHex: "b".repeat(64),
  }),
  parsePrivateKey: () => new Uint8Array(64),
  signNonce: () => "c".repeat(128),
}));

vi.mock("@/lib/telemetry", () => ({
  trackEvent: () => {},
}));

function renderPage(page: ReactElement): string {
  return renderToStaticMarkup(page);
}

function expectText(html: string, expected: string[]): void {
  expected.forEach((text) => {
    expect(html).toContain(text);
  });
}

describe("app render regressions", () => {
  beforeEach(() => {
    navigation.params = {};
    navigation.pathname = "/home";
    navigation.push.mockClear();
    navigation.redirect.mockClear();
  });

  it("renders workspace navigation around the home hub", () => {
    const html = renderPage(createElement(HomePage));

    expectText(html, [
      "HexRelay",
      "Home",
      "Servers",
      "Contacts",
      "Settings",
      "Onboarding complete",
      "No personas yet",
    ]);
    expect(html).toContain('aria-current="page"');
  });

  it("renders the identity onboarding entry state", () => {
    const html = renderPage(createElement(IdentityOnboardingPage));

    expectText(html, [
      "HexRelay onboarding",
      "Set up your local identity",
      "Identity bootstrap",
      "Create identity",
      "Import identity",
      "Continue to recovery",
    ]);
  });

  it("renders contacts session-required state inside the workspace shell", () => {
    navigation.pathname = "/contacts";
    const html = renderPage(createElement(ContactsPage));

    expectText(html, [
      "Contacts",
      "All contacts",
      "Requests",
      "Invites",
      "Add contact",
      "Share invite",
      "Search contacts",
      "Create or select a profile before managing contacts.",
    ]);
  });

  it("renders private-message session-required state for a routed contact", () => {
    navigation.pathname = "/contacts/usr-test-bob/messages";
    navigation.params = { contactId: "usr-test-bob" };
    const html = renderPage(createElement(ContactMessagesPage));

    expectText(html, [
      "Private Chat",
      "Back to contacts",
      "usr-test-bob",
      "Create or select a profile before messaging contacts.",
      "This node-routed private-message surface will send E2EE envelopes through the server delivery path.",
    ]);
  });
});
