import type { DragEvent, KeyboardEvent, MouseEvent, Ref } from "react";

import type { WorkspaceTab as OpenTab } from "@/lib/workspace-tabs";

import { Item } from "./item";
import type { Placement } from "./types";
import styles from "./styles.module.css";

type ListProps = {
  activeTabId?: string;
  collapsed: boolean;
  draggedTabId: string | null;
  emptyMessage: string;
  onCloseTab: (tab: OpenTab) => void;
  onContextMenu: (event: MouseEvent<HTMLElement>, tab: OpenTab) => void;
  onDragEnd: () => void;
  onDragStart: (tab: OpenTab, event: DragEvent<HTMLElement>) => void;
  onDrop: (tab: OpenTab, event: DragEvent<HTMLElement>) => void;
  onKeyboardContextMenu: (event: KeyboardEvent<HTMLElement>, tab: OpenTab) => void;
  tabListRef?: Ref<HTMLDivElement>;
  tabs: OpenTab[];
  variant: Placement;
};

export function List({
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
}: ListProps) {
  if (tabs.length === 0) {
    return emptyMessage ? <p className={styles.empty}>{emptyMessage}</p> : null;
  }

  return (
    <div
      className={styles.list}
      data-open-tab-collapsed={collapsed}
      data-open-tab-placement={variant}
      ref={tabListRef}
      role="list"
    >
      {tabs.map((tab) => (
        <Item
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
