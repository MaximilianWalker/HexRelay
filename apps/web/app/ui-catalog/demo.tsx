"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import { IconCircleCheck, IconMenu2, IconPalette, IconSearch, IconX } from "@tabler/icons-react";

import { BrandLockup } from "@/components/brand-lockup";
import { Badge } from "@/components/ui/display/badge";
import { Button } from "@/components/ui/buttons/button";
import { Dialog } from "@/components/ui/overlays/dialog";
import { DialogActions } from "@/components/ui/overlays/dialog-actions";
import { EmptyState } from "@/components/ui/feedback/empty-state";
import { Field } from "@/components/ui/forms/field";
import { IconButton } from "@/components/ui/buttons/icon-button";
import { PressableButton } from "@/components/ui/buttons/pressable-button";
import { SelectField } from "@/components/ui/forms/select-field";
import { ScrollArea } from "@/components/ui/navigation/scroll-area";
import { TextInput } from "@/components/ui/forms/text-input";
import {
  readThemePreference,
  setThemePreference,
  subscribeThemePreference,
  type ThemePreference,
} from "@/lib/ui/theme";

import {
  getGroupIdForSectionId,
  getSectionIdFromHash,
  matchesSection,
  sectionGroups,
  themeOptions,
  type Filter,
  type SectionGroupId,
  type SectionId,
  type ToggleGroupState,
  type VisibleSectionGroup,
} from "./data";
import { NavGroups } from "./nav";
import { DataDisplaySections } from "./sections/data-display";
import { FeedbackSections } from "./sections/feedback";
import { IdentitySections } from "./sections/identity";
import { InputsControlsSections } from "./sections/inputs-controls";
import { NavigationActionsSections } from "./sections/navigation-actions";
import { OverlaysSections, type PopupContent, type PopupHorizontal, type PopupVertical } from "./sections/overlays";
import { SurfacesSections } from "./sections/surfaces";

import styles from "./styles.module.css";

export function Demo() {
  const [toggleGroup, setToggleGroup] = useState<ToggleGroupState>("list");
  const [filter, setFilter] = useState<Filter>("muted");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [popupContent, setPopupContent] = useState<PopupContent>("panel");
  const [popupHorizontal, setPopupHorizontal] = useState<PopupHorizontal>("center");
  const [popupOpen, setPopupOpen] = useState(true);
  const [popupVertical, setPopupVertical] = useState<PopupVertical>("bottom");
  const [switchOff, setSwitchOff] = useState(false);
  const [switchOn, setSwitchOn] = useState(true);
  const [pinned, setPinned] = useState(true);
  const [navOpen, setNavOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [activeSectionId, setActiveSectionId] = useState<SectionId>("logo");
  const [openGroupIds, setOpenGroupIds] = useState<ReadonlySet<SectionGroupId>>(() => {
    return new Set([getGroupIdForSectionId("logo")]);
  });

  const themePreference = useSyncExternalStore<ThemePreference>(
    subscribeThemePreference,
    readThemePreference,
    () => "system",
  );
  const searchQuery = search.trim().toLowerCase();
  const searchActive = searchQuery.length > 0;
  const visibleSectionGroups: readonly VisibleSectionGroup[] = searchQuery
    ? sectionGroups
        .map((group) => ({
          id: group.id,
          label: group.label,
          sections: group.sections.filter((section) => matchesSection(section, group, searchQuery)),
        }))
        .filter((group) => group.sections.length > 0)
    : sectionGroups;
  const visibleSections = visibleSectionGroups.flatMap((group) => group.sections);
  const visibleSectionIds = new Set<SectionId>(visibleSections.map((section) => section.id));

  function isSectionVisible(sectionId: SectionId): boolean {
    return visibleSectionIds.has(sectionId);
  }

  function navigateSection(sectionId: SectionId): void {
    setActiveSectionId(sectionId);
    setOpenGroupIds((currentGroupIds) => {
      const nextGroupIds = new Set(currentGroupIds);
      nextGroupIds.add(getGroupIdForSectionId(sectionId));

      return nextGroupIds;
    });
    setNavOpen(false);
  }

  useEffect(() => {
    function scrollToHash(): void {
      const sectionId = getSectionIdFromHash(window.location.hash);
      if (!sectionId) {
        return;
      }

      setActiveSectionId(sectionId);
      setOpenGroupIds((currentGroupIds) => {
        const nextGroupIds = new Set(currentGroupIds);
        nextGroupIds.add(getGroupIdForSectionId(sectionId));

        return nextGroupIds;
      });
      window.requestAnimationFrame(() => {
        const target = document.getElementById(sectionId);

        if (typeof target?.scrollIntoView === "function") {
          target.scrollIntoView({ block: "start" });
        }
      });
    }

    scrollToHash();
    window.addEventListener("hashchange", scrollToHash);

    return () => window.removeEventListener("hashchange", scrollToHash);
  }, []);

  useEffect(() => {
    if (!navOpen) {
      return;
    }

    function closeOnEscape(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        setNavOpen(false);
      }
    }

    window.addEventListener("keydown", closeOnEscape);

    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [navOpen]);

  useEffect(() => {
    if (!window.matchMedia) {
      return;
    }

    const wideQuery = window.matchMedia("(min-width: 1281px)");

    function closeOnWide(event: MediaQueryList | MediaQueryListEvent): void {
      if (event.matches) {
        setNavOpen(false);
      }
    }

    closeOnWide(wideQuery);
    wideQuery.addEventListener("change", closeOnWide);

    return () => wideQuery.removeEventListener("change", closeOnWide);
  }, []);

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerMain}>
          <span className={styles.navToggle}>
            <Button
              aria-controls="catalog-nav-overlay"
              aria-expanded={navOpen}
              icon={<IconMenu2 aria-hidden="true" />}
              onClick={() => setNavOpen(true)}
              size="lg"
            >
              Catalog
            </Button>
          </span>
          <BrandLockup className={styles.headerBrand} size="sm" />
          <h1>UI Catalog</h1>
          <div className={styles.searchWrap}>
            <IconSearch aria-hidden="true" className={styles.searchIcon} />
            <TextInput
              aria-label="Search components"
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search components"
              type="search"
              value={search}
            />
          </div>
        </div>
        <div className={styles.headerActions}>
          <Field
            className={styles.themeField}
            label={
              <span className={styles.themeLabel}>
                <IconPalette aria-hidden="true" />
                <span>Theme</span>
              </span>
            }
          >
            <SelectField
              onChange={(event) => setThemePreference(event.target.value as ThemePreference)}
              value={themePreference}
            >
              {themeOptions.map((themeOption) => (
                <option key={themeOption.value} value={themeOption.value}>
                  {themeOption.label}
                </option>
              ))}
            </SelectField>
          </Field>
          <Badge tone="accent" icon={<IconCircleCheck aria-hidden="true" />} size="lg">
            Shared APIs
          </Badge>
        </div>
      </header>

      <div className={styles.shell}>
        <aside className={styles.nav} aria-label="UI catalog sections">
          <p className={styles.navTitle}>Components</p>
          <nav aria-label="UI catalog categories">
            <NavGroups
              activeSectionId={activeSectionId}
              groups={visibleSectionGroups}
              navId="catalog-sidebar"
              onExpandedChange={setOpenGroupIds}
              onNavigate={navigateSection}
              openGroupIds={openGroupIds}
              searchActive={searchActive}
            />
          </nav>
        </aside>

        <ScrollArea className={styles.catalogScrollArea} overlay viewportClassName={styles.content} width={4}>
          {visibleSections.length === 0 ? (
            <div className={styles.catalogEmptyState}>
              <EmptyState title="No components found">
                <p>Try a different component name or category.</p>
              </EmptyState>
            </div>
          ) : null}

          <IdentitySections isVisible={isSectionVisible} />
          <InputsControlsSections
            filter={filter}
            isVisible={isSectionVisible}
            onFilterChange={setFilter}
            onPinnedChange={setPinned}
            onSwitchOffChange={setSwitchOff}
            onSwitchOnChange={setSwitchOn}
            onToggleGroupChange={setToggleGroup}
            pinned={pinned}
            switchOff={switchOff}
            switchOn={switchOn}
            toggleGroup={toggleGroup}
          />
          <NavigationActionsSections isVisible={isSectionVisible} />
          <DataDisplaySections isVisible={isSectionVisible} />
          <FeedbackSections isVisible={isSectionVisible} />
          <SurfacesSections
            filter={filter}
            isVisible={isSectionVisible}
            onFilterChange={setFilter}
            onToggleGroupChange={setToggleGroup}
            toggleGroup={toggleGroup}
          />
          <OverlaysSections
            isVisible={isSectionVisible}
            onDialogOpenChange={setDialogOpen}
            onPopupContentChange={setPopupContent}
            onPopupHorizontalChange={setPopupHorizontal}
            onPopupOpenChange={setPopupOpen}
            onPopupVerticalChange={setPopupVertical}
            popupContent={popupContent}
            popupHorizontal={popupHorizontal}
            popupOpen={popupOpen}
            popupVertical={popupVertical}
          />
        </ScrollArea>
      </div>

      {navOpen ? (
        <div className={styles.navOverlay}>
          <PressableButton
            aria-label="Close catalog navigation"
            className={styles.navBackdrop}
            onClick={() => setNavOpen(false)}
            type="button"
          />
          <div
            aria-labelledby="catalog-nav-title"
            aria-modal="true"
            className={styles.navPanel}
            id="catalog-nav-overlay"
            role="dialog"
          >
            <div className={styles.navPanelHeader}>
              <p className={styles.navTitle} id="catalog-nav-title">
                Catalog navigation
              </p>
              <IconButton label="Close catalog navigation" onClick={() => setNavOpen(false)} size="sm">
                <IconX aria-hidden="true" />
              </IconButton>
            </div>
            <nav aria-label="UI catalog categories">
              <NavGroups
                activeSectionId={activeSectionId}
                groups={visibleSectionGroups}
                navId="catalog-overlay"
                onExpandedChange={setOpenGroupIds}
                onNavigate={navigateSection}
                openGroupIds={openGroupIds}
                searchActive={searchActive}
              />
            </nav>
          </div>
        </div>
      ) : null}

      {dialogOpen ? (
        <Dialog
          description="Dialog content uses the shared modal frame and action row."
          onClose={() => setDialogOpen(false)}
          title="Example dialog"
        >
          <p className={styles.dialogCopy}>Use this pattern for focused confirmation or short editing flows.</p>
          <DialogActions>
            <Button onClick={() => setDialogOpen(false)}>Cancel</Button>
            <Button data-autofocus onClick={() => setDialogOpen(false)} variant="primary">
              Confirm
            </Button>
          </DialogActions>
        </Dialog>
      ) : null}
    </main>
  );
}
