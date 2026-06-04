import { IconArrowsExchange, IconChevronRight, IconFocusCentered, IconMicrophone } from "@tabler/icons-react";

import { ButtonGroup } from "@/components/ui/button-group";
import { Menu as UiMenu, MenuItem } from "@/components/ui/menu";
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
  function selectNavLayout(nextLayout: NavLayout): void {
    onSetNavLayout(nextLayout);
    onClose();
  }

  return (
    <UiMenu
      aria-label="Profile actions menu"
      className={styles.menu}
      data-placement={placement}
      id="profile-more-menu"
      position="absolute"
      role="dialog"
    >
      <MenuItem
        className={styles.menuItem}
        icon={<IconFocusCentered className={styles.menuIcon} aria-hidden="true" />}
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

      <div className={styles.layoutItem}>
        <IconArrowsExchange className={styles.menuIcon} aria-hidden="true" />
        <span>Navigation</span>
        <div className={styles.layoutChoices}>
          <ButtonGroup
            label="Navigation layout"
            onChange={selectNavLayout}
            options={navLayoutOptions}
            value={navLayout}
          />
        </div>
      </div>

      <MenuItem
        className={styles.menuItem}
        icon={<IconMicrophone className={styles.menuIcon} aria-hidden="true" />}
        onClick={() => {
          onOpenAudioDevices();
          onClose();
        }}
        role="button"
        trailing={<IconChevronRight className={styles.menuChevron} aria-hidden="true" />}
      >
        Audio devices
      </MenuItem>
    </UiMenu>
  );
}
