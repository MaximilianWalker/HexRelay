import { Menu, type Item as Entry } from "@/components/ui/navigation/menu";

import type { SectionGroupId, SectionId, VisibleSectionGroup } from "./data";

import styles from "./styles.module.css";

export function NavGroups({
  activeSectionId,
  groups,
  navId,
  onExpandedChange,
  onNavigate,
  openGroupIds,
  searchActive,
}: {
  activeSectionId: SectionId;
  groups: readonly VisibleSectionGroup[];
  navId: string;
  onExpandedChange: (groupIds: ReadonlySet<SectionGroupId>) => void;
  onNavigate: (sectionId: SectionId) => void;
  openGroupIds: ReadonlySet<SectionGroupId>;
  searchActive: boolean;
}) {
  if (groups.length === 0) {
    return <p className={styles.navEmpty}>No matching components</p>;
  }

  const items: Entry[] = groups.map((group) => ({
    id: group.id,
    items: group.sections.map((section) => ({
      href: `#${section.id}`,
      id: section.id,
      name: section.label,
      onSelect: () => onNavigate(section.id),
    })),
    name: group.label,
  }));
  const expandedGroupIds = [...openGroupIds];
  const forceExpandedGroupIds = searchActive ? groups.map((group) => group.id) : [];

  return (
    <Menu
      activeId={activeSectionId}
      activeIndicator="rail"
      aria-label={`${navId} sections`}
      expandedIds={expandedGroupIds}
      forceExpandedIds={forceExpandedGroupIds}
      idleBorder={false}
      items={items}
      onExpandedChange={(nextExpandedIds) => {
        if (!searchActive) {
          onExpandedChange(new Set(nextExpandedIds as SectionGroupId[]));
        }
      }}
      panel={false}
    />
  );
}
