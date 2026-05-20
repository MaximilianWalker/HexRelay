export const HOME_ROUTE = "/home";
export const SERVERS_ROUTE = "/servers";
export const CONTACTS_ROUTE = "/contacts";
export const SETTINGS_ROUTE = "/settings";

function encodeSegment(value: string): string {
  return encodeURIComponent(value.trim());
}

export function serverWorkspaceRoute(serverId: string): string {
  return `${SERVERS_ROUTE}/${encodeSegment(serverId)}`;
}

export function dmWorkspaceRoute(contactId: string): string {
  return `${CONTACTS_ROUTE}/${encodeSegment(contactId)}/messages`;
}

export function isTopLevelMobileRoute(pathname: string): boolean {
  return [HOME_ROUTE, SERVERS_ROUTE, CONTACTS_ROUTE, SETTINGS_ROUTE].includes(pathname);
}
