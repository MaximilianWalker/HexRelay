import { IconArrowsExchange, IconChevronRight, IconFocusCentered, IconMicrophone } from "@tabler/icons-react";

import { SegmentedControl } from "@/components/ui/segmented-control";
import type { NavLayout } from "@/lib/workspace-preferences";
import { cx } from "@/lib/ui/cx";

import type { WorkspaceProfilePlacement } from "./workspace-profile-types";
import styles from "./workspace-profile-menu.module.css";

type WorkspaceProfileMenuProps = {
  collapsed: boolean;
  navLayout: NavLayout;
  onClose: () => void;
  onOpenAudioDevices: () => void;
  onSetCollapsed: (collapsed: boolean) => void;
  onSetNavLayout: (layout: NavLayout) => void;
  placement: WorkspaceProfilePlacement;
};

const navLayoutOptions: Array<{ id: NavLayout; label: string }> = [
  { id: "sidebar", label: "Sidebar" },
  { id: "topbar", label: "Topbar" },
];

export function WorkspaceProfileMenu({
  collapsed,
  navLayout,
  onClose,
  onOpenAudioDevices,
  onSetCollapsed,
  onSetNavLayout,
  placement,
}: WorkspaceProfileMenuProps) {
  function selectNavLayout(nextLayout: NavLayout): void {
    onSetNavLayout(nextLayout);
    onClose();
  }

  return (
    <div aria-label="Profile actions menu" className={styles.menu} data-placement={placement} id="profile-more-menu" role="dialog">
      <button
        aria-pressed={collapsed}
        className={styles.menuItem}
        onClick={() => onSetCollapsed(!collapsed)}
        type="button"
      >
        <IconFocusCentered className={styles.menuIcon} aria-hidden="true" />
        <span>Compact mode</span>
        <span className={cx(styles.switch, collapsed && styles.switchOn)} aria-hidden="true">
          <span />
        </span>
      </button>

      <div className={styles.layoutItem}>
        <IconArrowsExchange className={styles.menuIcon} aria-hidden="true" />
        <span>Navigation</span>
        <div className={styles.layoutChoices}>
          <SegmentedControl
            label="Navigation layout"
            onChange={selectNavLayout}
            options={navLayoutOptions}
            value={navLayout}
          />
        </div>
      </div>

      <button
        className={styles.menuItem}
        onClick={() => {
          onOpenAudioDevices();
          onClose();
        }}
        type="button"
      >
        <IconMicrophone className={styles.menuIcon} aria-hidden="true" />
        <span>Audio devices</span>
        <IconChevronRight className={styles.menuChevron} aria-hidden="true" />
      </button>
    </div>
  );
}
