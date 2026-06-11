import type { Ref, ReactNode } from "react";
import { IconChevronLeft, IconChevronRight, type Icon } from "@tabler/icons-react";

import { PressableButton } from "@/components/ui/buttons/pressable-button";
import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

export type Item = {
  id: string;
  label: string;
  icon?: Icon;
  onSelect?: () => void;
};

type BarProps = {
  activeId: string;
  activeRef?: Ref<HTMLElement>;
  actions?: ReactNode;
  canScrollLeft: boolean;
  canScrollRight: boolean;
  items: Item[];
  label: string;
  listRef: Ref<HTMLDivElement>;
  onChange?: (id: string) => void;
  onScrollLeft: () => void;
  onScrollRight: () => void;
};

export function Bar({
  activeId,
  activeRef,
  actions,
  canScrollLeft,
  canScrollRight,
  items,
  label,
  listRef,
  onChange,
  onScrollLeft,
  onScrollRight,
}: BarProps) {
  const hasItems = items.length > 0;

  if (!hasItems && !actions) {
    return null;
  }

  return (
    <div className={cx(styles.bar, !hasItems && styles.actionsOnly)}>
      {hasItems && canScrollLeft ? (
        <button
          aria-label="Scroll tabs left"
          className={styles.scrollButton}
          data-content-tab-scroll-button="left"
          onClick={onScrollLeft}
          type="button"
        >
          <IconChevronLeft className={styles.scrollIcon} aria-hidden="true" />
        </button>
      ) : null}
      {hasItems ? (
        <div aria-label={label} className={styles.tabs} ref={listRef}>
          {items.map((item) => {
            const TabIcon = item.icon;
            const active = item.id === activeId;
            const handleSelect = item.onSelect ?? (onChange ? () => onChange(item.id) : undefined);
            const className = cx(styles.item, handleSelect && styles.button, active && styles.active);
            const tabContent = (
              <>
                {TabIcon ? <TabIcon className={styles.icon} aria-hidden="true" /> : null}
                <span className={styles.label}>{item.label}</span>
              </>
            );

            return handleSelect ? (
              <PressableButton
                className={className}
                data-content-tab-id={item.id}
                key={item.id}
                onClick={handleSelect}
                pressed={active}
                ref={active ? (activeRef as Ref<HTMLButtonElement>) : undefined}
              >
                {tabContent}
              </PressableButton>
            ) : (
              <div
                className={className}
                data-content-tab-id={item.id}
                key={item.id}
                ref={active ? (activeRef as Ref<HTMLDivElement>) : undefined}
              >
                {tabContent}
              </div>
            );
          })}
        </div>
      ) : null}
      {hasItems && canScrollRight ? (
        <button
          aria-label="Scroll tabs right"
          className={styles.scrollButton}
          data-content-tab-scroll-button="right"
          onClick={onScrollRight}
          type="button"
        >
          <IconChevronRight className={styles.scrollIcon} aria-hidden="true" />
        </button>
      ) : null}
      {actions ? <div className={styles.actions}>{actions}</div> : null}
    </div>
  );
}
