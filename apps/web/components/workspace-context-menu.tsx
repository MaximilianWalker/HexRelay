import { IconPinned, IconPinnedOff, IconX } from "@tabler/icons-react";

import { Menu, MenuItem } from "@/components/ui/menu";
import type { WorkspaceTab } from "@/lib/workspace-tabs";

type WorkspaceContextMenuProps = {
  onCloseTab: (tab: WorkspaceTab) => void;
  onTogglePinned: (tab: WorkspaceTab) => void;
  position: { x: number; y: number };
  tab: WorkspaceTab;
};

export function WorkspaceContextMenu({ onCloseTab, onTogglePinned, position, tab }: WorkspaceContextMenuProps) {
  return (
    <Menu onClick={(event) => event.stopPropagation()} position="fixed" style={{ left: position.x, top: position.y }}>
      <MenuItem
        icon={tab.pinned ? <IconPinnedOff aria-hidden="true" /> : <IconPinned aria-hidden="true" />}
        onClick={() => onTogglePinned(tab)}
      >
        {tab.pinned ? "Unpin tab" : "Pin tab"}
      </MenuItem>
      <MenuItem icon={<IconX aria-hidden="true" />} onClick={() => onCloseTab(tab)} tone="danger">
        Close tab
      </MenuItem>
    </Menu>
  );
}
