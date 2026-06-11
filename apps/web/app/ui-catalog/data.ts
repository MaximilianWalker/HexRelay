import { THEME_OPTIONS, type ThemePreference } from "@/lib/ui/theme";

export type ToggleGroupState = "list" | "cards" | "disabled";
export type Filter = "all" | "unread" | "muted";

const themeLabels: Record<ThemePreference, string> = {
  dark: "Dark",
  light: "Light",
  system: "System",
};

export const themeOptions = THEME_OPTIONS.map((theme) => ({ label: themeLabels[theme], value: theme }));

export const sectionGroups = [
  {
    id: "identity",
    label: "Identity",
    sections: [{ id: "logo", label: "Logo", keywords: "brand mark lockup wordmark hexrelay" }],
  },
  {
    id: "inputs-controls",
    label: "Inputs & Controls",
    sections: [
      { id: "buttons", label: "Buttons", keywords: "button icon action press" },
      { id: "toggles", label: "Toggles", keywords: "switch segmented pressed control" },
      { id: "forms", label: "Forms", keywords: "field input select textarea checkbox" },
    ],
  },
  {
    id: "navigation-actions",
    label: "Navigation & Actions",
    sections: [
      { id: "list", label: "List", keywords: "list row item popover" },
      { id: "menu", label: "Menu", keywords: "menu nav row channel action submenu sidebar" },
      { id: "scroll-area", label: "Scroll Area", keywords: "scroll viewport overflow custom scrollbar" },
    ],
  },
  {
    id: "data-display",
    label: "Data Display",
    sections: [
      { id: "avatars", label: "Avatars", keywords: "user profile entity" },
      { id: "badges", label: "Badges", keywords: "label count status" },
    ],
  },
  {
    id: "feedback",
    label: "Feedback",
    sections: [
      { id: "alerts", label: "Alerts", keywords: "notice success danger warning" },
      { id: "empty-states", label: "Empty States", keywords: "empty blank fallback" },
    ],
  },
  {
    id: "surfaces",
    label: "Surfaces",
    sections: [
      { id: "panels", label: "Panels", keywords: "card container raised subtle" },
      { id: "toolbar", label: "Toolbar", keywords: "toolbar action row controls" },
    ],
  },
  {
    id: "overlays",
    label: "Overlays",
    sections: [
      { id: "dialogs", label: "Dialogs", keywords: "modal confirmation" },
      { id: "popups", label: "Popups", keywords: "popover floating anchored placement" },
    ],
  },
  {
    id: "app-patterns",
    label: "App Patterns",
    sections: [
      { id: "messages", label: "Messages", keywords: "chat message timeline composer channel rail presence" },
      { id: "profile-controls", label: "Profile Controls", keywords: "profile actions microphone sound compact menu" },
      { id: "content-tabs", label: "Content Tabs", keywords: "tabs scroll open workspace content" },
      { id: "settings-rows", label: "Settings Rows", keywords: "settings row panel status value control" },
      { id: "hub-surfaces", label: "Hub Surfaces", keywords: "servers contacts hubs cards list bulk actions" },
      { id: "workspace-rows", label: "Workspace Rows", keywords: "server workspace members voice participant icon" },
      { id: "contacts", label: "Contacts", keywords: "friend request discovery contact cards" },
    ],
  },
] as const;

export type SectionGroup = (typeof sectionGroups)[number];
export type SectionGroupId = SectionGroup["id"];
export type SectionEntry = SectionGroup["sections"][number];
export type SectionId = SectionEntry["id"];
export type VisibleSectionGroup = {
  id: SectionGroupId;
  label: SectionGroup["label"];
  sections: readonly SectionEntry[];
};

function getSectionsForGroup(group: SectionGroup): readonly SectionEntry[] {
  return group.sections;
}

export const sections: readonly SectionEntry[] = sectionGroups.flatMap(getSectionsForGroup);
const sectionIds = new Set<string>(sections.map((section) => section.id));

export function getSectionIdFromHash(hash: string): SectionId | null {
  const id = hash.slice(1);

  return sectionIds.has(id) ? (id as SectionId) : null;
}

export function getGroupIdForSectionId(sectionId: SectionId): SectionGroupId {
  const group = sectionGroups.find((item) => item.sections.some((section) => section.id === sectionId));

  return group?.id ?? "identity";
}

export function matchesSection(section: SectionEntry, group: SectionGroup, query: string): boolean {
  return `${group.label} ${group.id} ${section.label} ${section.id} ${section.keywords}`.toLowerCase().includes(query);
}
