import { IconArrowsExchange, IconChevronRight, IconFocusCentered, IconMicrophone } from "@tabler/icons-react";

import { ToggleGroup } from "@/components/ui/toggles/toggle-group";
import { List, ListButton, ListRow } from "@/components/ui/navigation/list";
import { Popup, type PopupPlacement } from "@/components/ui/overlays/popup";
import type { NavLayout } from "@/lib/workspace-preferences";
import { cx } from "@/lib/ui/cx";

import type { Placement } from "./types";
import styles from "./menu.module.css";

type MenuProps = {
  collapsed: boolean;
  navLayout: NavLayout;
  onClose: () => void;
  onOpenAudioDevices: () => void;
  onSetCollapsed: (collapsed: boolean) => void;
  onSetNavLayout: (layout: NavLayout) => void;
  placement: Placement;
};

const navLayoutOptions: Array<{ id: NavLayout; label: string }> = [
  { id: "sidebar", label: "Sidebar" },
  { id: "topbar", label: "Topbar" },
];

export function Menu({
  collapsed,
  navLayout,
  onClose,
  onOpenAudioDevices,
  onSetCollapsed,
  onSetNavLayout,
  placement,
}: MenuProps) {
  const menuPlacement: PopupPlacement = placement === "topbar" ? "bottom-end" : "right-end";

  function selectNavLayout(nextLayout: NavLayout): void {
    onSetNavLayout(nextLayout);
    onClose();
  }

  return (
    <Popup placement={menuPlacement}>
      <List aria-label="Profile actions menu" id="profile-more-menu" role="dialog">
        <ListButton
          icon={<IconFocusCentered aria-hidden="true" />}
          name="Compact mode"
          onClick={() => onSetCollapsed(!collapsed)}
          pressed={collapsed}
          role="button"
          end={
            <span className={cx(styles.switch, collapsed && styles.switchOn)} aria-hidden="true">
              <span />
            </span>
          }
        />

        <ListRow
          icon={<IconArrowsExchange aria-hidden="true" />}
          name="Navigation"
          end={
            <div className={styles.layoutChoices}>
              <ToggleGroup
                label="Navigation layout"
                onChange={selectNavLayout}
                options={navLayoutOptions}
                value={navLayout}
              />
            </div>
          }
        />

        <ListButton
          icon={<IconMicrophone aria-hidden="true" />}
          name="Audio devices"
          onClick={() => {
            onOpenAudioDevices();
            onClose();
          }}
          role="button"
          end={<IconChevronRight aria-hidden="true" />}
        />
      </List>
    </Popup>
  );
}
