"use client";

import { useRef } from "react";
import type { ReactNode } from "react";
import { IconCheck, IconVolume, IconVolumeOff } from "@tabler/icons-react";

import type { HubLayout } from "@/lib/hub-state";

import styles from "@/app/surfaces.module.css";

export type HubItem = {
  id: string;
  name: string;
  unread: number;
  pinned: boolean;
  muted: boolean;
};

export function HubSurface<T extends HubItem>({
  items,
  layout,
  selecting,
  selectedIds,
  noun,
  renderBadges,
  renderActions,
  onOpen,
  onToggleSelected,
}: {
  items: T[];
  layout: HubLayout;
  selecting: boolean;
  selectedIds: Set<string>;
  noun: "server" | "contact";
  renderBadges?: (item: T) => ReactNode;
  renderActions?: (item: T) => ReactNode;
  onOpen: (item: T) => void;
  onToggleSelected: (itemId: string) => void;
}) {
  const longPressRef = useRef<number | null>(null);

  function clearLongPress(): void {
    if (longPressRef.current !== null) {
      window.clearTimeout(longPressRef.current);
      longPressRef.current = null;
    }
  }

  function startLongPress(itemId: string): void {
    clearLongPress();
    longPressRef.current = window.setTimeout(() => {
      onToggleSelected(itemId);
      longPressRef.current = null;
    }, 420);
  }

  return (
    <div className={layout === "cards" ? styles.hubGrid : styles.hubList} role="list">
      {items.map((item) => {
        const selected = selectedIds.has(item.id);

        return (
          <article
            className={`${styles.hubItem} ${layout === "list" ? styles.hubItemList : ""} ${
              selected ? styles.hubItemSelected : ""
            }`}
            key={item.id}
            onPointerCancel={clearLongPress}
            onPointerDown={() => startLongPress(item.id)}
            onPointerLeave={clearLongPress}
            onPointerUp={clearLongPress}
            role="listitem"
          >
            {selecting ? (
              <button
                aria-label={selected ? `Deselect ${item.name}` : `Select ${item.name}`}
                aria-pressed={selected}
                className={`${styles.selectButton} ${selected ? styles.selectButtonActive : ""}`}
                onClick={() => onToggleSelected(item.id)}
                type="button"
              >
                {selected ? <IconCheck className={styles.icon} aria-hidden="true" /> : null}
              </button>
            ) : null}

            <button
              className={styles.hubItemMain}
              onClick={() => (selecting ? onToggleSelected(item.id) : onOpen(item))}
              type="button"
            >
              <span className={styles.avatar}>{initials(item.name)}</span>
              <span className={styles.hubItemText}>
                <span className={styles.title}>{item.name}</span>
                <span className={styles.meta}>
                  {item.unread > 0 ? `${item.unread} unread` : `No unread ${noun === "server" ? "channels" : "messages"}`}
                </span>
              </span>
            </button>

            <div className={styles.hubBadges}>
              {item.pinned ? <span className={styles.badge}>Pinned</span> : null}
              <span className={item.muted ? styles.badgeMuted : styles.badge}>
                {item.muted ? (
                  <IconVolumeOff className={styles.icon} aria-hidden="true" />
                ) : (
                  <IconVolume className={styles.icon} aria-hidden="true" />
                )}
                {item.muted ? "Muted" : "Audible"}
              </span>
              {renderBadges?.(item)}
            </div>

            {renderActions ? <div className={styles.hubActions}>{renderActions(item)}</div> : null}
          </article>
        );
      })}
    </div>
  );
}

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return "?";
  }

  return parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}
