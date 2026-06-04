import { IconArrowsExchange, IconChevronRight, IconFocusCentered, IconMicrophone } from "@tabler/icons-react";

import { ButtonGroup } from "@/components/ui/button-group";
import { Menu as UiMenu, MenuItem, MenuRow } from "@/components/ui/menu";
import { Popup, type PopupPlacement } from "@/components/ui/popup";
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
      <UiMenu aria-label="Profile actions menu" id="profile-more-menu" role="dialog">
        <MenuItem
          icon={<IconFocusCentered aria-hidden="true" />}
          onClick={() => onSetCollapsed(!collapsed)}
          pressed={collapsed}
          role="button"
          trailing={
            <span className={cx(styles.switch, collapsed && styles.switchOn)} aria-hidden="true">
              <span />
            </span>
          }
        >
          Compact mode
        </MenuItem>

        <MenuRow
          icon={<IconArrowsExchange aria-hidden="true" />}
          trailing={
            <div className={styles.layoutChoices}>
              <ButtonGroup
                label="Navigation layout"
                onChange={selectNavLayout}
                options={navLayoutOptions}
                value={navLayout}
              />
            </div>
          }
        >
          Navigation
        </MenuRow>

        <MenuItem
          icon={<IconMicrophone aria-hidden="true" />}
          onClick={() => {
            onOpenAudioDevices();
            onClose();
          }}
          role="button"
          trailing={<IconChevronRight aria-hidden="true" />}
        >
          Audio devices
        </MenuItem>
      </UiMenu>
    </Popup>
  );
}
