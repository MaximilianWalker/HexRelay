import type { DragEvent, KeyboardEvent, MouseEvent, Ref, WheelEvent } from "react";

import { ScrollButton } from "@/components/ui/navigation/scroll-button";
import type { WorkspaceTab as OpenTab } from "@/lib/workspace-tabs";

import { List } from "./list";
import type { Placement } from "./types";
import styles from "./styles.module.css";

type RootProps = {
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
  onScrollTabs?: (direction: -1 | 1) => void;
  onWheel?: (event: WheelEvent<HTMLElement>) => void;
  pinnedTabs: OpenTab[];
  regularTabs: OpenTab[];
  scrollState?: {
    canScrollLeft: boolean;
    canScrollRight: boolean;
  };
  tabListRef?: Ref<HTMLDivElement>;
  variant: Placement;
};

export function Root({
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
  onScrollTabs,
  onWheel,
  pinnedTabs,
  regularTabs,
  scrollState,
  tabListRef,
  variant,
}: RootProps) {
  if (variant === "topbar") {
    const tabs = [...pinnedTabs, ...regularTabs];

    return (
      <div className={styles.rail} onWheel={onWheel} role="group" aria-label="Open tabs">
        {tabs.length === 0 ? (
          <p className={styles.empty}>{emptyMessage}</p>
        ) : (
          <>
            {scrollState?.canScrollLeft ? (
              <ScrollButton
                direction="previous"
                label="Scroll open tabs left"
                onClick={() => onScrollTabs?.(-1)}
              />
            ) : null}
            <List
              activeTabId={activeTabId}
              collapsed={collapsed}
              draggedTabId={draggedTabId}
              emptyMessage={emptyMessage}
              onCloseTab={onCloseTab}
              onContextMenu={onContextMenu}
              onDragEnd={onDragEnd}
              onDragStart={onDragStart}
              onDrop={onDrop}
              onKeyboardContextMenu={onKeyboardContextMenu}
              tabListRef={tabListRef}
              tabs={tabs}
              variant={variant}
            />
            {scrollState?.canScrollRight ? (
              <ScrollButton
                direction="next"
                label="Scroll open tabs right"
                onClick={() => onScrollTabs?.(1)}
              />
            ) : null}
          </>
        )}
      </div>
    );
  }

  const showRegularTabs = regularTabs.length > 0 || !collapsed;

  return (
    <>
      {pinnedTabs.length > 0 ? (
        <div
          className={`${styles.section} ${styles.pinnedSection}`}
          data-collapsed={collapsed}
          role="group"
          aria-label="Pinned tabs"
        >
          <List
            activeTabId={activeTabId}
            collapsed={collapsed}
            draggedTabId={draggedTabId}
            emptyMessage=""
            onCloseTab={onCloseTab}
            onContextMenu={onContextMenu}
            onDragEnd={onDragEnd}
            onDragStart={onDragStart}
            onDrop={onDrop}
            onKeyboardContextMenu={onKeyboardContextMenu}
            tabs={pinnedTabs}
            variant={variant}
          />
        </div>
      ) : null}
      {showRegularTabs ? (
        <div className={styles.section} data-collapsed={collapsed} role="group" aria-label="Open tabs">
          <List
            activeTabId={activeTabId}
            collapsed={collapsed}
            draggedTabId={draggedTabId}
            emptyMessage={emptyMessage}
            onCloseTab={onCloseTab}
            onContextMenu={onContextMenu}
            onDragEnd={onDragEnd}
            onDragStart={onDragStart}
            onDrop={onDrop}
            onKeyboardContextMenu={onKeyboardContextMenu}
            tabs={regularTabs}
            variant={variant}
          />
        </div>
      ) : null}
    </>
  );
}
