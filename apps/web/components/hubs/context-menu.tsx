"use client";

import { IconCheck, IconExternalLink } from "@tabler/icons-react";

import { List, ListButton } from "@/components/ui/list";
import { Popup } from "@/components/ui/popup";

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
    <Popup position="fixed" style={{ left: x, top: y }}>
      <List onClick={(event) => event.stopPropagation()} role="menu">
        <ListButton
          icon={<IconExternalLink aria-hidden="true" />}
          name={`Open ${noun}`}
          onClick={onOpen}
          role="menuitem"
        />
        <ListButton
          icon={<IconCheck aria-hidden="true" />}
          name={selected ? "Deselect" : "Select"}
          onClick={onToggleSelected}
          role="menuitem"
        />
      </List>
    </Popup>
  );
}
