"use client";

import { IconCheck, IconExternalLink } from "@tabler/icons-react";

import styles from "./hubs.module.css";

export function HubContextMenu({
  noun,
  onOpen,
  onToggleSelected,
  selected,
  x,
  y,
}: {
  noun: "server" | "contact";
  onOpen: () => void;
  onToggleSelected: () => void;
  selected: boolean;
  x: number;
  y: number;
}) {
  return (
    <div
      className={styles.contextMenu}
      onClick={(event) => event.stopPropagation()}
      role="menu"
      style={{ left: x, top: y }}
    >
      <button className={styles.contextMenuItem} onClick={onOpen} role="menuitem" type="button">
        <IconExternalLink className={styles.icon} aria-hidden="true" />
        Open {noun}
      </button>
      <button className={styles.contextMenuItem} onClick={onToggleSelected} role="menuitem" type="button">
        <IconCheck className={styles.icon} aria-hidden="true" />
        {selected ? "Deselect" : "Select"}
      </button>
    </div>
  );
}
