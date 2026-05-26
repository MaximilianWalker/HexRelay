export const THEME_STORAGE_KEY = "hexrelay.ui.theme";
const THEME_EVENT = "hexrelay-ui-theme-changed";

export const THEME_OPTIONS = ["system", "light", "dark"] as const;

export type ThemePreference = (typeof THEME_OPTIONS)[number];

export const CUSTOM_THEME_VARIABLES = [
  "--color-bg-app",
  "--color-surface",
  "--color-surface-subtle",
  "--color-surface-raised",
  "--color-surface-glass",
  "--color-surface-muted",
  "--color-surface-selected",
  "--color-border",
  "--color-border-subtle",
  "--color-border-strong",
  "--color-border-control",
  "--color-border-danger",
  "--color-text",
  "--color-text-strong",
  "--color-text-heading",
  "--color-text-muted",
  "--color-text-subtle",
  "--color-text-inverse",
  "--color-accent",
  "--color-accent-strong",
  "--color-accent-muted",
  "--color-accent-border",
  "--color-success",
  "--color-success-muted",
  "--color-warning",
  "--color-warning-muted",
  "--color-danger",
  "--color-danger-strong",
  "--color-danger-muted",
  "--color-info-muted",
  "--color-focus",
  "--color-backdrop",
] as const;

export type CustomThemeVariable = (typeof CUSTOM_THEME_VARIABLES)[number];
export type CustomThemeTokens = Partial<Record<CustomThemeVariable, string>>;

const CUSTOM_THEME_VARIABLE_SET = new Set<string>(CUSTOM_THEME_VARIABLES);

export function parseThemePreference(value: string | null | undefined): ThemePreference {
  return value === "light" || value === "dark" || value === "system" ? value : "system";
}

export function readThemePreference(): ThemePreference {
  if (typeof window === "undefined") {
    return "system";
  }

  try {
    return parseThemePreference(window.localStorage.getItem(THEME_STORAGE_KEY));
  } catch {
    return "system";
  }
}

export function subscribeThemePreference(onChange: () => void): () => void {
  if (typeof window === "undefined") {
    return () => {};
  }

  function handleStorage(event: StorageEvent): void {
    if (event.key === THEME_STORAGE_KEY) {
      onChange();
    }
  }

  window.addEventListener("storage", handleStorage);
  window.addEventListener(THEME_EVENT, onChange);

  return () => {
    window.removeEventListener("storage", handleStorage);
    window.removeEventListener(THEME_EVENT, onChange);
  };
}

export function setThemePreference(theme: ThemePreference): void {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // The current document can still reflect the setting when storage is blocked.
  }

  applyThemePreference(theme);
  window.dispatchEvent(new Event(THEME_EVENT));
}

export function sanitizeCustomThemeTokens(tokens: Record<string, string>): CustomThemeTokens {
  const sanitized: CustomThemeTokens = {};

  for (const [key, value] of Object.entries(tokens)) {
    if (!CUSTOM_THEME_VARIABLE_SET.has(key)) {
      continue;
    }

    const trimmed = value.trim();
    if (!trimmed || /[;{}]/.test(trimmed)) {
      continue;
    }

    sanitized[key as CustomThemeVariable] = trimmed;
  }

  return sanitized;
}

export function applyThemePreference(theme: ThemePreference, root: HTMLElement = document.documentElement): void {
  if (theme === "system") {
    root.dataset.theme = "system";
    return;
  }

  root.dataset.theme = theme;
}
