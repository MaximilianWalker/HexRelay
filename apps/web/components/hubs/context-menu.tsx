"use client";

import { IconCheck, IconExternalLink } from "@tabler/icons-react";

import { Menu, MenuItem } from "@/components/ui/menu";
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
      <Menu onClick={(event) => event.stopPropagation()}>
        <MenuItem icon={<IconExternalLink aria-hidden="true" />} onClick={onOpen}>
          Open {noun}
        </MenuItem>
        <MenuItem icon={<IconCheck aria-hidden="true" />} onClick={onToggleSelected}>
          {selected ? "Deselect" : "Select"}
        </MenuItem>
      </Menu>
    </Popup>
  );
}
