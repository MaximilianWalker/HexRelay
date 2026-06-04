import Link from "next/link";
import type { DragEvent, KeyboardEvent, MouseEvent } from "react";
import { IconX } from "@tabler/icons-react";

import { Avatar } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { IconButton } from "@/components/ui/icon-button";
import { cx } from "@/lib/ui/cx";
import { initials } from "@/lib/ui/initials";
import type { WorkspaceTab as OpenTab } from "@/lib/workspace-tabs";

import styles from "./item.module.css";

type ItemProps = {
  active: boolean;
  dragging: boolean;
  onClose: (tab: OpenTab) => void;
  onContextMenu: (event: MouseEvent<HTMLElement>, tab: OpenTab) => void;
  onDragEnd: () => void;
  onDragStart: (tab: OpenTab, event: DragEvent<HTMLElement>) => void;
  onDrop: (tab: OpenTab, event: DragEvent<HTMLElement>) => void;
  onKeyboardContextMenu: (event: KeyboardEvent<HTMLElement>, tab: OpenTab) => void;
  tab: OpenTab;
};

function unreadCount(value: number | undefined): number {
  if (!Number.isFinite(value) || !value || value <= 0) {
    return 0;
  }

  return Math.floor(value);
}

export function Item({
  active,
  dragging,
  onClose,
  onContextMenu,
  onDragEnd,
  onDragStart,
  onDrop,
  onKeyboardContextMenu,
  tab,
}: ItemProps) {
  const imageLabel = tab.imageLabel ?? tab.label;
  const isServer = tab.kind === "server";
  const unread = unreadCount(tab.unread);

  return (
    <div
      className={cx(styles.tab, active && styles.active, tab.pinned && styles.pinned)}
      data-open-tab-id={tab.id}
      draggable
      onContextMenu={(event) => onContextMenu(event, tab)}
      onDragEnd={onDragEnd}
      onDragOver={(event) => {
        if (dragging) {
          event.preventDefault();
        }
      }}
      onDragStart={(event) => onDragStart(tab, event)}
      onDrop={(event) => {
        event.preventDefault();
        onDrop(tab, event);
      }}
      role="listitem"
    >
      <Link
        aria-current={active ? "page" : undefined}
        aria-label={`${tab.kind === "dm" ? "Conversation" : "Server"}: ${tab.label}`}
        className={styles.link}
        href={tab.href}
        onKeyDown={(event) => onKeyboardContextMenu(event, tab)}
      >
        <Avatar
          className={cx(styles.image, isServer ? styles.imageServer : styles.imageContact)}
          aria-hidden="true"
          kind={isServer ? "server" : "user"}
          text={initials(imageLabel)}
        />
        <span className={styles.label}>{tab.label}</span>
      </Link>
      <div className={styles.actions}>
        {isServer && unread > 0 ? (
          <Badge aria-label={`${unread} unread notifications`} shape="counter" size="sm" tone="accent">
            {unread}
          </Badge>
        ) : null}
        {!tab.pinned ? (
          <IconButton
            label={`Close ${tab.label}`}
            onClick={() => onClose(tab)}
            size="sm"
            title="Close tab"
            variant="ghost"
          >
            <IconX aria-hidden="true" />
          </IconButton>
        ) : null}
      </div>
    </div>
  );
}
