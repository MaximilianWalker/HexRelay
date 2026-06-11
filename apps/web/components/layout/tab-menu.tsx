import { IconPinned, IconPinnedOff, IconX } from "@tabler/icons-react";

import { List, ListButton } from "@/components/ui/navigation/list";
import { Popup } from "@/components/ui/overlays/popup";
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
      <List onClick={(event) => event.stopPropagation()} role="menu">
        <ListButton
          icon={tab.pinned ? <IconPinnedOff aria-hidden="true" /> : <IconPinned aria-hidden="true" />}
          name={tab.pinned ? "Unpin tab" : "Pin tab"}
          onClick={() => onTogglePinned(tab)}
          role="menuitem"
        />
        <ListButton
          icon={<IconX aria-hidden="true" />}
          name="Close tab"
          onClick={() => onCloseTab(tab)}
          role="menuitem"
          tone="danger"
        />
      </List>
    </Popup>
  );
}
