import { describe, expect, it } from "vitest";

import {
  CONTACTS_ROUTE,
  HOME_ROUTE,
  SETTINGS_ROUTE,
  dmWorkspaceRoute,
  isTopLevelMobileRoute,
  serverWorkspaceRoute,
} from "./navigation-routes";

describe("navigation routes", () => {
  it("builds stable top-level and workspace routes", () => {
    expect(HOME_ROUTE).toBe("/home");
    expect(CONTACTS_ROUTE).toBe("/contacts");
    expect(SETTINGS_ROUTE).toBe("/settings");
    expect(serverWorkspaceRoute("core team")).toBe("/servers/core%20team");
    expect(dmWorkspaceRoute("usr:nora")).toBe("/contacts/usr%3Anora/messages");
  });

  it("identifies the mobile top-level routes only", () => {
    expect(isTopLevelMobileRoute("/servers")).toBe(true);
    expect(isTopLevelMobileRoute("/contacts/usr-nora/messages")).toBe(false);
  });
});
