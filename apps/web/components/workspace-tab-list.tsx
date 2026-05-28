import type { DragEvent, KeyboardEvent, MouseEvent, Ref } from "react";

import type { WorkspaceTab } from "@/lib/workspace-tabs";

import { WorkspaceTabItem } from "./workspace-tab-item";
import type { WorkspaceTabPlacement } from "./workspace-tab-types";
import styles from "./workspace-tabs.module.css";

type WorkspaceTabListProps = {
  activeTabId?: string;
  collapsed: boolean;
  draggedTabId: string | null;
  emptyMessage: string;
  onCloseTab: (tab: WorkspaceTab) => void;
  onContextMenu: (event: MouseEvent<HTMLElement>, tab: WorkspaceTab) => void;
  onDragEnd: () => void;
  onDragStart: (tab: WorkspaceTab, event: DragEvent<HTMLElement>) => void;
  onDrop: (tab: WorkspaceTab, event: DragEvent<HTMLElement>) => void;
  onKeyboardContextMenu: (event: KeyboardEvent<HTMLElement>, tab: WorkspaceTab) => void;
  tabListRef?: Ref<HTMLDivElement>;
  tabs: WorkspaceTab[];
  variant: WorkspaceTabPlacement;
};

export function WorkspaceTabList({
  activeTabId,
  collapsed,
  draggedTabId,
  emptyMessage,
  onCloseTab,
  onContextMenu,
  onDragEnd,
  onDragStart,
  onDrop,
  onKeyboardContextMenu,
  tabListRef,
  tabs,
  variant,
}: WorkspaceTabListProps) {
  if (tabs.length === 0) {
    return emptyMessage ? <p className={styles.empty}>{emptyMessage}</p> : null;
  }

  return (
    <div
      className={styles.list}
      data-workspace-tab-collapsed={collapsed}
      data-workspace-tab-placement={variant}
      ref={tabListRef}
      role="list"
    >
      {tabs.map((tab) => (
        <WorkspaceTabItem
          active={activeTabId === tab.id}
          dragging={Boolean(draggedTabId)}
          key={tab.id}
          onClose={onCloseTab}
          onContextMenu={onContextMenu}
          onDragEnd={onDragEnd}
          onDragStart={onDragStart}
          onDrop={onDrop}
          onKeyboardContextMenu={onKeyboardContextMenu}
          tab={tab}
        />
      ))}
    </div>
  );
}
