"use client";

import { useEffect, useSyncExternalStore } from "react";

import {
  applyThemePreference,
  readThemePreference,
  subscribeThemePreference,
  type ThemePreference,
} from "@/lib/ui/theme";

export function ThemeController() {
  const theme = useSyncExternalStore<ThemePreference>(
    subscribeThemePreference,
    readThemePreference,
    () => "system",
  );

  useEffect(() => {
    applyThemePreference(theme);
  }, [theme]);

  return null;
}
