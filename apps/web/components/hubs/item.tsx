"use client";

import type { PointerEvent, ReactNode } from "react";
import { IconCheck, IconVolume, IconVolumeOff } from "@tabler/icons-react";

import { Avatar } from "@/components/ui/display/avatar";
import { Badge } from "@/components/ui/display/badge";
import { PressableButton } from "@/components/ui/buttons/pressable-button";
import { cx } from "@/lib/ui/cx";
import { initials } from "@/lib/ui/initials";

import type { ItemData } from "./surface";
import styles from "./styles.module.css";

export function Item<T extends ItemData>({
  item,
  layout,
  noun,
  onContextMenu,
  onOpen,
  onPointerCancel,
  onPointerDown,
  onPointerLeave,
  onPointerUp,
  renderActions,
  renderBadges,
  selected,
  selecting,
}: {
  item: T;
  layout: "cards" | "list";
  noun: "server" | "contact";
  onContextMenu: (event: React.MouseEvent<HTMLElement>) => void;
  onOpen: () => void;
  onPointerCancel: () => void;
  onPointerDown: (event: PointerEvent<HTMLElement>) => void;
  onPointerLeave: () => void;
  onPointerUp: () => void;
  renderActions?: (item: T) => ReactNode;
  renderBadges?: (item: T) => ReactNode;
  selected: boolean;
  selecting: boolean;
}) {
  const kind = noun === "server" ? "server" : "user";

  return (
    <article
      className={cx(styles.hubItem, layout === "list" && styles.hubItemList, selected && styles.hubItemSelected)}
      onContextMenu={onContextMenu}
      onPointerCancel={onPointerCancel}
      onPointerDown={onPointerDown}
      onPointerLeave={onPointerLeave}
      onPointerUp={onPointerUp}
      role="listitem"
    >
      {selecting ? (
        <span aria-hidden="true" className={cx(styles.selectIndicator, selected && styles.selectIndicatorActive)}>
          {selected ? <IconCheck className={styles.icon} aria-hidden="true" /> : null}
        </span>
      ) : null}

      <PressableButton
        className={styles.hubItemMain}
        data-hub-main
        onClick={onOpen}
        pressed={selecting ? selected : undefined}
      >
        <Avatar kind={kind} text={initials(item.name)} />
        <span className={styles.hubItemText}>
          <span className={styles.hubItemTitle}>{item.name}</span>
          <span className={styles.hubItemMeta}>
            {item.unread > 0 ? `${item.unread} unread` : `No unread ${noun === "server" ? "channels" : "messages"}`}
          </span>
        </span>
      </PressableButton>

      <div className={styles.hubBadges}>
        {item.pinned ? <Badge tone="accent">Pinned</Badge> : null}
        <Badge icon={item.muted ? <IconVolumeOff aria-hidden="true" /> : <IconVolume aria-hidden="true" />} tone={item.muted ? "muted" : "neutral"}>
          {item.muted ? "Muted" : "Audible"}
        </Badge>
        {renderBadges?.(item)}
      </div>

      {renderActions ? (
        <div className={styles.hubActions} data-hub-actions>
          {renderActions(item)}
        </div>
      ) : null}
    </article>
  );
}
