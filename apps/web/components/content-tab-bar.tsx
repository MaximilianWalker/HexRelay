import type { Ref, ReactNode } from "react";
import { IconChevronLeft, IconChevronRight, type Icon } from "@tabler/icons-react";

import { PressableButton } from "@/components/ui/pressable-button";
import { cx } from "@/lib/ui/cx";

import styles from "./content-tab-bar.module.css";

export type ContentTabItem = {
  id: string;
  label: string;
  icon?: Icon;
  onSelect?: () => void;
};

type ContentTabBarProps = {
  activeTabId: string;
  canScrollLeft: boolean;
  canScrollRight: boolean;
  label: string;
  onScrollLeft: () => void;
  onScrollRight: () => void;
  onTabChange?: (tabId: string) => void;
  tabActions?: ReactNode;
  tabListRef: Ref<HTMLDivElement>;
  tabs: ContentTabItem[];
  activeTabRef?: Ref<HTMLElement>;
};

export function ContentTabBar({
  activeTabId,
  activeTabRef,
  canScrollLeft,
  canScrollRight,
  label,
  onScrollLeft,
  onScrollRight,
  onTabChange,
  tabActions,
  tabListRef,
  tabs,
}: ContentTabBarProps) {
  const hasTabs = tabs.length > 0;

  if (!hasTabs && !tabActions) {
    return null;
  }

  return (
    <div className={cx(styles.tabBar, !hasTabs && styles.actionBarOnly)}>
      {hasTabs && canScrollLeft ? (
        <button
          aria-label="Scroll tabs left"
          className={styles.scrollButton}
          data-tab-scroll-button="left"
          onClick={onScrollLeft}
          type="button"
        >
          <IconChevronLeft className={styles.scrollIcon} aria-hidden="true" />
        </button>
      ) : null}
      {hasTabs ? (
        <div aria-label={label} className={styles.tabs} ref={tabListRef}>
          {tabs.map((tab) => {
            const TabIcon = tab.icon;
            const active = tab.id === activeTabId;
            const handleTabSelect = tab.onSelect ?? (onTabChange ? () => onTabChange(tab.id) : undefined);
            const tabClassName = cx(styles.tab, handleTabSelect && styles.tabButton, active && styles.tabActive);
            const tabContent = (
              <>
                {TabIcon ? <TabIcon className={styles.tabIcon} aria-hidden="true" /> : null}
                <span className={styles.tabLabel}>{tab.label}</span>
              </>
            );

            return handleTabSelect ? (
              <PressableButton
                className={tabClassName}
                data-tab-id={tab.id}
                key={tab.id}
                onClick={handleTabSelect}
                pressed={active}
                ref={active ? (activeTabRef as Ref<HTMLButtonElement>) : undefined}
              >
                {tabContent}
              </PressableButton>
            ) : (
              <div
                className={tabClassName}
                data-tab-id={tab.id}
                key={tab.id}
                ref={active ? (activeTabRef as Ref<HTMLDivElement>) : undefined}
              >
                {tabContent}
              </div>
            );
          })}
        </div>
      ) : null}
      {hasTabs && canScrollRight ? (
        <button
          aria-label="Scroll tabs right"
          className={styles.scrollButton}
          data-tab-scroll-button="right"
          onClick={onScrollRight}
          type="button"
        >
          <IconChevronRight className={styles.scrollIcon} aria-hidden="true" />
        </button>
      ) : null}
      {tabActions ? <div className={styles.tabActions}>{tabActions}</div> : null}
    </div>
  );
}
