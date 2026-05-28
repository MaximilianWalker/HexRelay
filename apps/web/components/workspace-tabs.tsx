import type { DragEvent, KeyboardEvent, MouseEvent, Ref, WheelEvent } from "react";
import { IconChevronLeft, IconChevronRight } from "@tabler/icons-react";

import type { WorkspaceTab } from "@/lib/workspace-tabs";

import { WorkspaceTabList } from "./workspace-tab-list";
import type { WorkspaceTabPlacement } from "./workspace-tab-types";
import styles from "./workspace-tabs.module.css";

type WorkspaceTabsProps = {
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
  onScrollTabs?: (direction: -1 | 1) => void;
  onWheel?: (event: WheelEvent<HTMLElement>) => void;
  pinnedTabs: WorkspaceTab[];
  regularTabs: WorkspaceTab[];
  scrollState?: {
    canScrollLeft: boolean;
    canScrollRight: boolean;
  };
  tabListRef?: Ref<HTMLDivElement>;
  variant: WorkspaceTabPlacement;
};

export function WorkspaceTabs({
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
}: WorkspaceTabsProps) {
  if (variant === "topbar") {
    const tabs = [...pinnedTabs, ...regularTabs];

    return (
      <div className={styles.rail} onWheel={onWheel} role="group" aria-label="Workspace tabs">
        {tabs.length === 0 ? (
          <p className={styles.empty}>{emptyMessage}</p>
        ) : (
          <>
            {scrollState?.canScrollLeft ? (
              <button
                aria-label="Scroll workspace tabs left"
                className={styles.scrollButton}
                onClick={() => onScrollTabs?.(-1)}
                type="button"
              >
                <IconChevronLeft className={styles.scrollIcon} aria-hidden="true" />
              </button>
            ) : null}
            <WorkspaceTabList
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
              <button
                aria-label="Scroll workspace tabs right"
                className={styles.scrollButton}
                onClick={() => onScrollTabs?.(1)}
                type="button"
              >
                <IconChevronRight className={styles.scrollIcon} aria-hidden="true" />
              </button>
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
          <WorkspaceTabList
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
        <div className={styles.section} data-collapsed={collapsed} role="group" aria-label="Workspace tabs">
          <WorkspaceTabList
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
