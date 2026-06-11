"use client";

import { IconChevronDown, IconChevronRight } from "@tabler/icons-react";
import type { AriaAttributes, CSSProperties, HTMLAttributes, ReactNode } from "react";
import { useState } from "react";

import { cx } from "@/lib/ui/cx";

import { List, ListButton, ListLink, type ListIconColor, type ListSize, type ListTone } from "./list";
import styles from "./menu.module.css";

type IdCollection = Iterable<string>;
type MenuRootElement = "div" | "nav";
type MenuSkin = "default" | "sidebar";
type MenuActiveIndicator = "rail" | "none";
type MenuSpacing = "sm" | "md";
type MenuCurrent = AriaAttributes["aria-current"] | boolean;

export type Item = {
  ariaLabel?: string;
  current?: MenuCurrent;
  disabled?: boolean;
  end?: ReactNode;
  href?: string;
  icon?: ReactNode;
  iconColor?: ListIconColor;
  id: string;
  items?: readonly Item[];
  name: ReactNode;
  onSelect?: () => void;
  pressed?: boolean;
  size?: ListSize;
  title?: string;
  tone?: ListTone;
};

type MenuProps = Omit<HTMLAttributes<HTMLElement>, "children" | "onSelect"> & {
  activeId?: string;
  activeIndicator?: MenuActiveIndicator;
  as?: MenuRootElement;
  collapsed?: boolean;
  defaultExpandedIds?: IdCollection;
  empty?: ReactNode;
  expandedIds?: IdCollection;
  forceExpandedIds?: IdCollection;
  iconColor?: ListIconColor;
  idleBorder?: boolean;
  indentSubmenus?: boolean;
  items: readonly Item[];
  onExpandedChange?: (expandedIds: string[]) => void;
  onItemActivate?: (item: Item) => void;
  panel?: boolean;
  skin?: MenuSkin;
  spacing?: MenuSpacing;
};

function collectionToSet(collection: IdCollection | undefined): Set<string> {
  return new Set(collection ?? []);
}

function getItemLabel(item: Item, collapsed: boolean): string | undefined {
  if (!collapsed || item.ariaLabel || typeof item.name !== "string") {
    return item.ariaLabel;
  }

  return item.name;
}

function getItemTitle(item: Item, collapsed: boolean): string | undefined {
  if (item.title || typeof item.name !== "string") {
    return item.title;
  }

  return collapsed ? item.name : undefined;
}

function renderSubmenuEnd(item: Item, expanded: boolean) {
  return (
    <span className={styles.menuEndGroup}>
      {item.end}
      {expanded ? <IconChevronDown aria-hidden="true" /> : <IconChevronRight aria-hidden="true" />}
    </span>
  );
}

export function Menu({
  activeId,
  activeIndicator,
  as = "div",
  className,
  collapsed = false,
  defaultExpandedIds,
  empty,
  expandedIds,
  forceExpandedIds,
  iconColor = "default",
  idleBorder = true,
  indentSubmenus = true,
  items,
  onExpandedChange,
  onItemActivate,
  panel = false,
  role,
  skin = "default",
  spacing = "md",
  ...props
}: MenuProps) {
  const [uncontrolledExpandedIds, setUncontrolledExpandedIds] = useState(() => collectionToSet(defaultExpandedIds));
  const baseExpandedIds = expandedIds === undefined ? uncontrolledExpandedIds : collectionToSet(expandedIds);
  const effectiveExpandedIds = new Set([...baseExpandedIds, ...collectionToSet(forceExpandedIds)]);
  const resolvedActiveIndicator = activeIndicator ?? "rail";

  function setExpanded(nextExpandedIds: Set<string>): void {
    if (expandedIds === undefined) {
      setUncontrolledExpandedIds(nextExpandedIds);
    }

    onExpandedChange?.([...nextExpandedIds]);
  }

  function toggleItem(itemId: string): void {
    const nextExpandedIds = new Set(baseExpandedIds);

    if (nextExpandedIds.has(itemId)) {
      nextExpandedIds.delete(itemId);
    } else {
      nextExpandedIds.add(itemId);
    }

    setExpanded(nextExpandedIds);
  }

  function activateItem(item: Item): void {
    item.onSelect?.();
    onItemActivate?.(item);
  }

  function renderItems(entries: readonly Item[], depth: number): ReactNode {
    return entries.map((item) => {
      const hasSubmenu = Boolean(item.items?.length);
      const expanded = effectiveExpandedIds.has(item.id);
      const panelId = hasSubmenu ? `menu-${item.id}-submenu` : undefined;
      const active = item.id === activeId;
      const commonProps = {
        active,
        disabled: item.disabled,
        icon: item.icon,
        iconColor: item.iconColor ?? iconColor,
        name: item.name,
        size: item.size,
        title: getItemTitle(item, collapsed),
        tone: item.tone,
        "aria-label": getItemLabel(item, collapsed),
      };

      if (hasSubmenu) {
        return (
          <div
            className={styles.menuGroup}
            data-menu-depth={depth}
            key={item.id}
            style={{ "--menu-depth": depth } as CSSProperties}
          >
            <ListButton
              {...commonProps}
              aria-controls={panelId}
              aria-expanded={expanded}
              end={renderSubmenuEnd(item, expanded)}
              onClick={() => toggleItem(item.id)}
            />
            <div
              className={styles.menuSubmenu}
              hidden={!expanded}
              id={panelId}
              style={{ "--menu-depth": depth + 1 } as CSSProperties}
            >
              {renderItems(item.items ?? [], depth + 1)}
            </div>
          </div>
        );
      }

      if (item.href) {
        return (
          <ListLink
            {...commonProps}
            current={item.current ?? (active ? "page" : undefined)}
            end={item.end}
            href={item.href}
            key={item.id}
            onClick={() => activateItem(item)}
          />
        );
      }

      return (
        <ListButton
          {...commonProps}
          current={item.current}
          end={item.end}
          key={item.id}
          onClick={() => activateItem(item)}
          pressed={item.pressed}
        />
      );
    });
  }

  return (
    <List
      as={as}
      className={cx(styles.menu, className)}
      data-menu-active-indicator={resolvedActiveIndicator}
      data-menu-collapsed={collapsed ? "true" : undefined}
      data-menu-idle-border={idleBorder ? undefined : "hidden"}
      data-menu-indent-submenus={indentSubmenus ? "true" : undefined}
      data-menu-skin={skin === "default" ? undefined : skin}
      data-menu-spacing={spacing}
      panel={panel}
      role={role}
      {...props}
    >
      {items.length > 0 ? renderItems(items, 0) : empty}
    </List>
  );
}
