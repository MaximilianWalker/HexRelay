"use client";

import { useEffect, useRef, useState } from "react";
import type { MouseEvent, PointerEvent, ReactNode } from "react";

import type { HubLayout } from "@/lib/hub-state";

import { HubContextMenu } from "./hub-context-menu";
import { HubItem as HubItemComponent } from "./hub-item";
import styles from "./hubs.module.css";

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
  const suppressOpenRef = useRef<string | null>(null);
  const [contextMenu, setContextMenu] = useState<{ itemId: string; x: number; y: number } | null>(null);
  const contextItem = contextMenu ? items.find((item) => item.id === contextMenu.itemId) : undefined;

  useEffect(() => {
    if (!contextMenu) {
      return;
    }

    function closeMenu(): void {
      setContextMenu(null);
    }

    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        closeMenu();
      }
    }

    document.addEventListener("click", closeMenu);
    document.addEventListener("keydown", handleKeyDown);
    window.addEventListener("resize", closeMenu);
    window.addEventListener("scroll", closeMenu, true);

    return () => {
      document.removeEventListener("click", closeMenu);
      document.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("resize", closeMenu);
      window.removeEventListener("scroll", closeMenu, true);
    };
  }, [contextMenu]);

  function clearLongPress(): void {
    if (longPressRef.current !== null) {
      window.clearTimeout(longPressRef.current);
      longPressRef.current = null;
    }
  }

  function startLongPress(itemId: string, event: PointerEvent<HTMLElement>): void {
    if (event.button !== 0) {
      return;
    }

    const target = event.target instanceof HTMLElement ? event.target : null;
    if (target?.closest("[data-hub-actions]")) {
      return;
    }

    clearLongPress();
    longPressRef.current = window.setTimeout(() => {
      onToggleSelected(itemId);
      suppressOpenRef.current = itemId;
      longPressRef.current = null;
    }, 420);
  }

  function openContextMenu(event: MouseEvent<HTMLElement>, itemId: string): void {
    event.preventDefault();
    clearLongPress();
    setContextMenu({ itemId, x: event.clientX, y: event.clientY });
  }

  function handleOpen(item: T): void {
    if (suppressOpenRef.current === item.id) {
      suppressOpenRef.current = null;
      return;
    }

    if (selecting) {
      onToggleSelected(item.id);
      return;
    }

    onOpen(item);
  }

  function toggleFromContext(itemId: string): void {
    onToggleSelected(itemId);
    setContextMenu(null);
  }

  return (
    <div className={layout === "cards" ? styles.hubGrid : styles.hubList} role="list">
      {items.map((item) => {
        const selected = selectedIds.has(item.id);

        return (
          <HubItemComponent
            item={item}
            key={item.id}
            layout={layout}
            noun={noun}
            onContextMenu={(event) => openContextMenu(event, item.id)}
            onOpen={() => handleOpen(item)}
            onPointerCancel={clearLongPress}
            onPointerDown={(event) => startLongPress(item.id, event)}
            onPointerLeave={clearLongPress}
            onPointerUp={clearLongPress}
            renderActions={renderActions}
            renderBadges={renderBadges}
            selected={selected}
            selecting={selecting}
          />
        );
      })}
      {contextItem && contextMenu ? (
        <HubContextMenu
          noun={noun}
          onOpen={() => {
            onOpen(contextItem);
            setContextMenu(null);
          }}
          onToggleSelected={() => toggleFromContext(contextItem.id)}
          selected={selectedIds.has(contextItem.id)}
          x={contextMenu.x}
          y={contextMenu.y}
        />
      ) : null}
    </div>
  );
}
