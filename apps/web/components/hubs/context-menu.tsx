"use client";

import { IconCheck, IconExternalLink } from "@tabler/icons-react";

import { Menu, MenuItem } from "@/components/ui/menu";

import styles from "./styles.module.css";

export function ContextMenu({
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
    <Menu
      className={styles.contextMenu}
      onClick={(event) => event.stopPropagation()}
      position="fixed"
      style={{ left: x, top: y }}
    >
      <MenuItem icon={<IconExternalLink className={styles.icon} aria-hidden="true" />} onClick={onOpen}>
        Open {noun}
      </MenuItem>
      <MenuItem icon={<IconCheck className={styles.icon} aria-hidden="true" />} onClick={onToggleSelected}>
        {selected ? "Deselect" : "Select"}
      </MenuItem>
    </Menu>
  );
}
