import { IconPinned, IconPinnedOff, IconX } from "@tabler/icons-react";

import { Menu, MenuItem } from "@/components/ui/menu";
import { Popup } from "@/components/ui/popup";
import type { WorkspaceTab as OpenTab } from "@/lib/workspace-tabs";

type TabMenuProps = {
  onCloseTab: (tab: OpenTab) => void;
  onTogglePinned: (tab: OpenTab) => void;
  position: { x: number; y: number };
  tab: OpenTab;
};

export function TabMenu({ onCloseTab, onTogglePinned, position, tab }: TabMenuProps) {
  return (
    <Popup position="fixed" style={{ left: position.x, top: position.y }}>
      <Menu onClick={(event) => event.stopPropagation()}>
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
    </Popup>
  );
}
