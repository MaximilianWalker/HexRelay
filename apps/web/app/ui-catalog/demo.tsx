"use client";

import { useEffect, useState, type ReactNode } from "react";
import {
  IconAlertTriangle,
  IconBell,
  IconBellOff,
  IconCheck,
  IconChevronDown,
  IconCircleCheck,
  IconCircleX,
  IconHash,
  IconInfoCircle,
  IconLayoutGrid,
  IconList,
  IconMenu2,
  IconMessageCircle,
  IconPinned,
  IconPlus,
  IconSearch,
  IconServer2,
  IconSettings,
  IconTrash,
  IconUserPlus,
  IconVolume,
  IconX,
} from "@tabler/icons-react";

import { Avatar } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { ButtonGroup } from "@/components/ui/button-group";
import { Button } from "@/components/ui/button";
import { CheckboxField } from "@/components/ui/checkbox-field";
import { Dialog } from "@/components/ui/dialog";
import { DialogActions } from "@/components/ui/dialog-actions";
import { EmptyState } from "@/components/ui/empty-state";
import { Field } from "@/components/ui/field";
import { IconButton } from "@/components/ui/icon-button";
import { ListActionButton } from "@/components/ui/list-action-button";
import { Menu, MenuItem, MenuRow } from "@/components/ui/menu";
import { Alert } from "@/components/ui/alert";
import { Panel } from "@/components/ui/panel";
import { Popup, type PopupPlacement } from "@/components/ui/popup";
import { PressableButton } from "@/components/ui/pressable-button";
import { SelectField } from "@/components/ui/select-field";
import { TextArea } from "@/components/ui/text-area";
import { TextInput } from "@/components/ui/text-input";
import { ToggleButton } from "@/components/ui/toggle-button";
import { ToggleSwitch } from "@/components/ui/toggle-switch";

import styles from "./styles.module.css";

type ButtonGroupState = "list" | "cards" | "disabled";
type Filter = "all" | "unread" | "muted";
type PopupContent = "alert" | "menu" | "panel";
type PopupHorizontal = "center" | "left" | "right";
type PopupVertical = "bottom" | "center" | "top";

const sections = [
  { id: "buttons", label: "Buttons" },
  { id: "toggles", label: "Toggles" },
  { id: "menus", label: "Menus" },
  { id: "lists", label: "List Actions" },
  { id: "forms", label: "Forms" },
  { id: "badges", label: "Badges" },
  { id: "avatars", label: "Avatars" },
  { id: "alerts", label: "Alerts" },
  { id: "empty-states", label: "Empty States" },
  { id: "panels", label: "Panels" },
  { id: "dialogs", label: "Dialogs" },
  { id: "popups", label: "Popups" },
] as const;

const popupVerticalOptions: Array<{ label: string; value: PopupVertical }> = [
  { label: "Top", value: "top" },
  { label: "Center", value: "center" },
  { label: "Bottom", value: "bottom" },
];

const popupHorizontalOptions: Array<{ label: string; value: PopupHorizontal }> = [
  { label: "Left", value: "left" },
  { label: "Center", value: "center" },
  { label: "Right", value: "right" },
];

const popupContentOptions: Array<{ label: string; value: PopupContent }> = [
  { label: "Panel", value: "panel" },
  { label: "Menu", value: "menu" },
  { label: "Alert", value: "alert" },
];

function getPopupPlacement(vertical: PopupVertical, horizontal: PopupHorizontal): PopupPlacement {
  if (vertical === "center" && horizontal === "center") {
    return "center";
  }

  if (vertical === "center") {
    return horizontal === "left" ? "left-center" : "right-center";
  }

  if (horizontal === "left") {
    return `${vertical}-start`;
  }

  if (horizontal === "right") {
    return `${vertical}-end`;
  }

  return `${vertical}-center`;
}

function Section({
  children,
  description,
  hideHeader = false,
  id,
  title,
}: {
  children: ReactNode;
  description?: string;
  hideHeader?: boolean;
  id: string;
  title: string;
}) {
  return (
    <section aria-label={hideHeader ? title : undefined} className={styles.section} id={id}>
      {hideHeader ? null : (
        <div className={styles.sectionHeader}>
          <h2>{title}</h2>
          {description ? <p>{description}</p> : null}
        </div>
      )}
      {children}
    </section>
  );
}

function Example({
  children,
  title,
  wide,
}: {
  children: ReactNode;
  title: string;
  wide?: boolean;
}) {
  return (
    <div className={wide ? `${styles.example} ${styles.exampleWide}` : styles.example}>
      <h3>{title}</h3>
      {children}
    </div>
  );
}

function CatalogNavLinks({ onNavigate }: { onNavigate?: () => void }) {
  return (
    <>
      {sections.map((section) => (
        <a href={`#${section.id}`} key={section.id} onClick={onNavigate}>
          {section.label}
        </a>
      ))}
    </>
  );
}

function PopupPreviewContent({ content }: { content: PopupContent }) {
  if (content === "menu") {
    return (
      <Menu role="group">
        <MenuItem icon={<IconSettings aria-hidden="true" />} role="button">
          Open settings
        </MenuItem>
        <MenuItem icon={<IconBellOff aria-hidden="true" />} role="button">
          Mute activity
        </MenuItem>
      </Menu>
    );
  }

  if (content === "alert") {
    return (
      <Alert icon={<IconCircleCheck aria-hidden="true" />} tone="success">
        Activity synced
      </Alert>
    );
  }

  return (
    <Panel padding="sm" variant="raised">
      <p className={styles.popupText}>Atlas activity is synced.</p>
    </Panel>
  );
}

export function Demo() {
  const [buttonGroup, setButtonGroup] = useState<ButtonGroupState>("list");
  const [filter, setFilter] = useState<Filter>("muted");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [popupContent, setPopupContent] = useState<PopupContent>("panel");
  const [popupHorizontal, setPopupHorizontal] = useState<PopupHorizontal>("center");
  const [popupOpen, setPopupOpen] = useState(true);
  const [popupVertical, setPopupVertical] = useState<PopupVertical>("bottom");
  const [switchOff, setSwitchOff] = useState(false);
  const [switchOn, setSwitchOn] = useState(true);
  const [pinned, setPinned] = useState(true);
  const [catalogNavOpen, setCatalogNavOpen] = useState(false);

  const popupPlacement = getPopupPlacement(popupVertical, popupHorizontal);

  useEffect(() => {
    function scrollToHash(): void {
      const id = window.location.hash.slice(1);
      if (!id) {
        return;
      }

      window.requestAnimationFrame(() => {
        const target = document.getElementById(id);

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
    if (!catalogNavOpen) {
      return;
    }

    function closeOnEscape(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        setCatalogNavOpen(false);
      }
    }

    window.addEventListener("keydown", closeOnEscape);

    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [catalogNavOpen]);

  useEffect(() => {
    if (!window.matchMedia) {
      return;
    }

    const wideCatalogQuery = window.matchMedia("(min-width: 1281px)");

    function closeOnWideCatalog(event: MediaQueryList | MediaQueryListEvent): void {
      if (event.matches) {
        setCatalogNavOpen(false);
      }
    }

    closeOnWideCatalog(wideCatalogQuery);
    wideCatalogQuery.addEventListener("change", closeOnWideCatalog);

    return () => wideCatalogQuery.removeEventListener("change", closeOnWideCatalog);
  }, []);

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerMain}>
          <span className={styles.navToggle}>
            <IconButton
              aria-controls="catalog-nav-overlay"
              aria-expanded={catalogNavOpen}
              label="Open catalog navigation"
              onClick={() => setCatalogNavOpen(true)}
              size="lg"
            >
              <IconMenu2 aria-hidden="true" />
            </IconButton>
          </span>
          <h1>UI catalog</h1>
        </div>
        <span className={styles.headerBadge}>
          <Badge tone="accent" icon={<IconCircleCheck aria-hidden="true" />}>
            Shared APIs
          </Badge>
        </span>
      </header>

      <div className={styles.shell}>
        <aside className={styles.nav} aria-label="UI catalog sections">
          <p className={styles.navTitle}>Catalog</p>
          <nav>
            <CatalogNavLinks />
          </nav>
        </aside>

        <div className={styles.content}>
          <Section
            id="buttons"
            title="Buttons"
            description="One shared Button contract covers text, icon-text, icon-only, link, active, disabled, and loading states."
          >
            <div className={styles.buttonLayout}>
              <div className={styles.buttonStack}>
                <Example title="Variants">
                  <div className={styles.row}>
                    <Button variant="primary" icon={<IconPlus aria-hidden="true" />}>
                      Primary
                    </Button>
                    <Button variant="secondary" icon={<IconSettings aria-hidden="true" />}>
                      Secondary
                    </Button>
                    <Button variant="ghost" icon={<IconBell aria-hidden="true" />}>
                      Ghost
                    </Button>
                    <Button variant="danger" icon={<IconTrash aria-hidden="true" />}>
                      Danger
                    </Button>
                  </div>
                </Example>

                <Example title="Tones">
                  <div className={styles.row}>
                    <Button icon={<IconBell aria-hidden="true" />} tone="accent">
                      Accent
                    </Button>
                    <Button icon={<IconCircleCheck aria-hidden="true" />} tone="success">
                      Success
                    </Button>
                    <Button icon={<IconTrash aria-hidden="true" />} tone="danger">
                      Danger
                    </Button>
                    <Button icon={<IconBellOff aria-hidden="true" />} tone="muted">
                      Muted
                    </Button>
                  </div>
                </Example>

                <Example title="Content">
                  <div className={styles.row}>
                    <Button>Text only</Button>
                    <Button icon={<IconPinned aria-hidden="true" />}>With icon</Button>
                    <Button icon={<IconChevronDown aria-hidden="true" />} iconPosition="end">
                      End icon
                    </Button>
                    <IconButton label="Search">
                      <IconSearch aria-hidden="true" />
                    </IconButton>
                  </div>
                </Example>

                <Example title="States">
                  <div className={styles.row}>
                    <ToggleButton
                      icon={<IconPinned aria-hidden="true" />}
                      onPressedChange={setPinned}
                      pressed={pinned}
                    >
                      Pressed
                    </ToggleButton>
                    <Button disabled icon={<IconX aria-hidden="true" />}>
                      Disabled
                    </Button>
                  </div>
                </Example>
              </div>

              <Example title="Types and Sizes">
                <div className={styles.sizeTable}>
                  <span className={styles.sizeTableCorner} aria-hidden="true" />
                  <span className={styles.sizeTableHeader}>Text</span>
                  <span className={styles.sizeTableHeader}>Icon + text</span>
                  <span className={styles.sizeTableHeader}>Icon only</span>

                  <span className={styles.sizeTableLabel}>Small</span>
                  <div className={styles.sizeTableCell}>
                    <Button size="sm">Small</Button>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <Button icon={<IconCheck aria-hidden="true" />} size="sm">
                      Small icon
                    </Button>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <IconButton label="Small search" size="sm">
                      <IconSearch aria-hidden="true" />
                    </IconButton>
                  </div>

                  <span className={styles.sizeTableLabel}>Medium</span>
                  <div className={styles.sizeTableCell}>
                    <Button>Medium</Button>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <Button icon={<IconPinned aria-hidden="true" />}>Medium icon</Button>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <IconButton label="Medium search">
                      <IconSearch aria-hidden="true" />
                    </IconButton>
                  </div>

                  <span className={styles.sizeTableLabel}>Large</span>
                  <div className={styles.sizeTableCell}>
                    <Button size="lg">Large</Button>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <Button icon={<IconServer2 aria-hidden="true" />} size="lg">
                      Large icon
                    </Button>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <IconButton label="Large search" size="lg">
                      <IconSearch aria-hidden="true" />
                    </IconButton>
                  </div>
                </div>
              </Example>
            </div>
          </Section>

          <Section
            id="toggles"
            title="Toggles"
            description="Toggle buttons, button groups, and switches share active color, font, weight, and icon sizing."
          >
            <div className={styles.buttonLayout}>
              <div className={styles.buttonStack}>
                <Example title="Toggle Buttons">
                  <div className={styles.row}>
                    <ToggleButton
                      icon={<IconMessageCircle aria-hidden="true" />}
                      onPressedChange={() => setFilter(filter === "unread" ? "all" : "unread")}
                      pressed={filter === "unread"}
                    >
                      Unread
                    </ToggleButton>
                    <ToggleButton
                      icon={<IconBellOff aria-hidden="true" />}
                      onPressedChange={() => setFilter(filter === "muted" ? "all" : "muted")}
                      pressed={filter === "muted"}
                    >
                      Muted
                    </ToggleButton>
                    <ToggleButton
                      icon={<IconPinned aria-hidden="true" />}
                      onPressedChange={() => setFilter(filter === "all" ? "muted" : "all")}
                      pressed={filter === "all"}
                    >
                      All
                    </ToggleButton>
                  </div>
                </Example>

                <Example title="Switches">
                  <div className={styles.row}>
                    <span className={styles.switchLabel}>Off</span>
                    <ToggleSwitch checked={switchOff} label="Off switch" onChange={setSwitchOff} />
                    <span className={styles.switchLabel}>On</span>
                    <ToggleSwitch checked={switchOn} label="On switch" onChange={setSwitchOn} />
                    <span className={styles.switchLabel}>Disabled</span>
                    <ToggleSwitch checked={false} disabled label="Disabled switch" />
                  </div>
                </Example>
              </div>

              <Example title="Sizes">
                <div className={styles.controlSizeList}>
                  <span className={styles.matrixLabel}>Small</span>
                  <ButtonGroup
                    label="Small view mode"
                    onChange={setButtonGroup}
                    options={[
                      { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
                      { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
                      { id: "disabled", label: "Disabled", disabled: true },
                    ]}
                    size="sm"
                    value={buttonGroup}
                  />
                  <span className={styles.matrixLabel}>Medium</span>
                  <ButtonGroup
                    label="Medium view mode"
                    onChange={setButtonGroup}
                    options={[
                      { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
                      { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
                      { id: "disabled", label: "Disabled", disabled: true },
                    ]}
                    value={buttonGroup}
                  />
                  <span className={styles.matrixLabel}>Large</span>
                  <ButtonGroup
                    label="Large view mode"
                    onChange={setButtonGroup}
                    options={[
                      { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
                      { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
                      { id: "disabled", label: "Disabled", disabled: true },
                    ]}
                    size="lg"
                    value={buttonGroup}
                  />
                </div>
              </Example>
            </div>
          </Section>

          <Section
            id="menus"
            title="Menus"
            description="Menus centralize item layout, keyboard focus movement, icon slots, danger tone, and item size."
          >
            <div className={styles.exampleGrid}>
              <Example title="Items">
                <Menu>
                  <MenuItem icon={<IconSettings aria-hidden="true" />}>Profile settings</MenuItem>
                  <MenuItem icon={<IconBellOff aria-hidden="true" />} pressed>
                    Mute notifications
                  </MenuItem>
                  <MenuItem icon={<IconUserPlus aria-hidden="true" />} trailing={<Badge tone="muted">New</Badge>}>
                    Invite contact
                  </MenuItem>
                  <MenuItem icon={<IconTrash aria-hidden="true" />} tone="danger">
                    Leave server
                  </MenuItem>
                </Menu>
              </Example>

              <Example title="Actions">
                <Menu role="group">
                  <MenuItem icon={<IconPinned aria-hidden="true" />} role="button">
                    Pin tab
                  </MenuItem>
                  <MenuItem disabled icon={<IconVolume aria-hidden="true" />} role="button">
                    Voice unavailable
                  </MenuItem>
                </Menu>
              </Example>

              <Example title="Static Content">
                <Menu role="group">
                  <MenuRow icon={<IconInfoCircle aria-hidden="true" />} trailing={<Badge tone="muted">Current</Badge>}>
                    Sidebar layout
                  </MenuRow>
                  <MenuItem icon={<IconSettings aria-hidden="true" />} role="button">
                    Edit preferences
                  </MenuItem>
                </Menu>
              </Example>

              <Example title="Sizes">
                <Menu role="group">
                  <MenuItem icon={<IconSettings aria-hidden="true" />} role="button" size="sm">
                    Small
                  </MenuItem>
                  <MenuItem icon={<IconPinned aria-hidden="true" />} role="button">
                    Medium
                  </MenuItem>
                  <MenuItem icon={<IconVolume aria-hidden="true" />} role="button" size="lg">
                    Large
                  </MenuItem>
                </Menu>
              </Example>
            </div>
          </Section>

          <Section
            id="lists"
            title="List Actions"
            description="Channel, hub, and action-list rows should use the shared list action recipe instead of local button styling."
          >
            <div className={styles.exampleGrid}>
              <Example title="States">
                <div className={styles.stack}>
                  <ListActionButton icon={<IconHash aria-hidden="true" />}>Default</ListActionButton>
                  <ListActionButton active icon={<IconMessageCircle aria-hidden="true" />}>
                    Active
                  </ListActionButton>
                  <ListActionButton badge="8" badgeLabel="8 unread" icon={<IconHash aria-hidden="true" />}>
                    With badge
                  </ListActionButton>
                  <ListActionButton disabled icon={<IconBellOff aria-hidden="true" />}>Disabled</ListActionButton>
                </div>
              </Example>

              <Example title="Sizes">
                <div className={styles.stack}>
                  <ListActionButton icon={<IconHash aria-hidden="true" />} size="sm">
                    Small
                  </ListActionButton>
                  <ListActionButton icon={<IconMessageCircle aria-hidden="true" />}>
                    Medium
                  </ListActionButton>
                  <ListActionButton icon={<IconServer2 aria-hidden="true" />} size="lg">
                    Large
                  </ListActionButton>
                </div>
              </Example>
            </div>
          </Section>

          <Section id="forms" title="Forms" description="Fields share label, helper, invalid, disabled, and control typography styles.">
            <div className={styles.exampleGrid}>
              <Example title="Text Inputs" wide>
                <div className={styles.fieldGrid}>
                  <Field helper="Shown in profile cards and mentions." label="Default">
                    <TextInput defaultValue="Diogo" />
                  </Field>
                  <Field label="Invalid" error="Server name is required.">
                    <TextInput invalid placeholder="Product team" />
                  </Field>
                  <Field helper="Locked by current role." label="Disabled">
                    <TextInput defaultValue="Read only" disabled />
                  </Field>
                </div>
              </Example>

              <Example title="Selects" wide>
                <div className={styles.fieldGrid}>
                  <Field helper="Visible to contacts." label="Default">
                    <SelectField defaultValue="online">
                      <option value="online">Online</option>
                      <option value="away">Away</option>
                      <option value="offline">Offline</option>
                    </SelectField>
                  </Field>
                  <Field label="Invalid" error="Pick a role before saving.">
                    <SelectField defaultValue="" invalid>
                      <option value="">Select role</option>
                      <option value="admin">Admin</option>
                      <option value="member">Member</option>
                    </SelectField>
                  </Field>
                  <Field helper="Managed by server policy." label="Disabled">
                    <SelectField defaultValue="member" disabled>
                      <option value="member">Member</option>
                    </SelectField>
                  </Field>
                </div>
              </Example>

              <Example title="Text Areas" wide>
                <div className={styles.fieldGrid}>
                  <Field label="Default">
                    <TextArea defaultValue="Share release notes, planning threads, and voice rooms here." rows={4} />
                  </Field>
                  <Field label="Invalid" error="Message is too long.">
                    <TextArea defaultValue="This note needs a shorter summary." invalid rows={4} />
                  </Field>
                  <Field helper="Generated from server settings." label="Disabled">
                    <TextArea defaultValue="Only admins can edit this note." disabled rows={4} />
                  </Field>
                </div>
              </Example>

              <Example title="Checkboxes">
                <div className={styles.column}>
                  <CheckboxField>Unchecked</CheckboxField>
                  <CheckboxField defaultChecked>Checked</CheckboxField>
                  <CheckboxField disabled>Disabled</CheckboxField>
                </div>
              </Example>
            </div>
          </Section>

          <Section id="badges" title="Badges" description="Badges expose semantic tones and optional icons without changing text metrics.">
            <div className={styles.exampleGrid}>
              <Example title="Types and Sizes" wide>
                <div className={styles.sizeTable}>
                  <span className={styles.sizeTableCorner} aria-hidden="true" />
                  <span className={styles.sizeTableHeader}>Text</span>
                  <span className={styles.sizeTableHeader}>Icon + text</span>
                  <span className={styles.sizeTableHeader}>Number</span>

                  <span className={styles.sizeTableLabel}>Small</span>
                  <div className={styles.sizeTableCell}>
                    <Badge size="sm">Small</Badge>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <Badge icon={<IconCircleCheck aria-hidden="true" />} size="sm" tone="success">
                      Small icon
                    </Badge>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <Badge shape="counter" size="sm" tone="accent">
                      8
                    </Badge>
                  </div>

                  <span className={styles.sizeTableLabel}>Medium</span>
                  <div className={styles.sizeTableCell}>
                    <Badge>Medium</Badge>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <Badge icon={<IconCircleCheck aria-hidden="true" />} tone="success">
                      Medium icon
                    </Badge>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <Badge shape="counter" tone="accent">12</Badge>
                  </div>

                  <span className={styles.sizeTableLabel}>Large</span>
                  <div className={styles.sizeTableCell}>
                    <Badge size="lg">Large</Badge>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <Badge icon={<IconCircleCheck aria-hidden="true" />} size="lg" tone="success">
                      Large icon
                    </Badge>
                  </div>
                  <div className={styles.sizeTableCell}>
                    <Badge shape="counter" size="lg" tone="accent">
                      24
                    </Badge>
                  </div>
                </div>
              </Example>

              <Example title="Tones" wide>
                <div className={styles.row}>
                  <Badge>Neutral</Badge>
                  <Badge tone="muted">Muted</Badge>
                  <Badge tone="accent">Accent</Badge>
                  <Badge tone="success">Success</Badge>
                  <Badge tone="warning">Warning</Badge>
                  <Badge tone="danger">Danger</Badge>
                </div>
              </Example>
            </div>
          </Section>

          <Section id="avatars" title="Avatars" description="Avatars expose user and server shapes with the same three supported sizes.">
            <div className={styles.exampleGrid}>
              <Example title="User Sizes" wide>
                <div className={styles.avatarGrid}>
                  <div className={styles.avatarSample}>
                    <Avatar label="Small user" size="sm" text="SM" />
                    <span className={styles.sampleLabel}>Small user</span>
                  </div>
                  <div className={styles.avatarSample}>
                    <Avatar label="Medium user" text="MD" />
                    <span className={styles.sampleLabel}>Medium user</span>
                  </div>
                  <div className={styles.avatarSample}>
                    <Avatar label="Large user" size="lg" text="LG" />
                    <span className={styles.sampleLabel}>Large user</span>
                  </div>
                </div>
              </Example>

              <Example title="Server Sizes" wide>
                <div className={styles.avatarGrid}>
                  <div className={styles.avatarSample}>
                    <Avatar kind="server" label="Small server" size="sm" text="SM" />
                    <span className={styles.sampleLabel}>Small server</span>
                  </div>
                  <div className={styles.avatarSample}>
                    <Avatar kind="server" label="Medium server" text="MD" />
                    <span className={styles.sampleLabel}>Medium server</span>
                  </div>
                  <div className={styles.avatarSample}>
                    <Avatar kind="server" label="Large server" size="lg" text="LG" />
                    <span className={styles.sampleLabel}>Large server</span>
                  </div>
                </div>
              </Example>
            </div>
          </Section>

          <Section id="alerts" title="Alerts" description="Alerts expose every semantic tone through the same layout and typography.">
            <div className={styles.exampleGrid}>
              <Example title="Tones" wide>
                <div className={styles.stack}>
                  <Alert icon={<IconInfoCircle aria-hidden="true" />}>Info alert</Alert>
                  <Alert icon={<IconCircleCheck aria-hidden="true" />} tone="success">
                    Success alert
                  </Alert>
                  <Alert icon={<IconAlertTriangle aria-hidden="true" />} tone="warning">
                    Warning alert
                  </Alert>
                  <Alert icon={<IconCircleX aria-hidden="true" />} tone="danger">
                    Danger alert
                  </Alert>
                </div>
              </Example>
            </div>
          </Section>

          <Section id="empty-states" title="Empty States" description="Empty states centralize title, copy, and optional action placement.">
            <div className={styles.exampleGrid}>
              <Example title="Default">
                <EmptyState title="No contacts yet">Contacts will appear here after requests are accepted.</EmptyState>
              </Example>

              <Example title="With Action">
                <EmptyState
                  action={
                    <Button icon={<IconPlus aria-hidden="true" />} variant="primary">
                      Create server
                    </Button>
                  }
                  title="No servers yet"
                >
                  Add a server to start testing channels, members, and voice rooms.
                </EmptyState>
              </Example>
            </div>
          </Section>

          <Section id="panels" title="Panels" description="Panels carry surface tone and padding variants without redefining controls.">
            <div className={styles.exampleGrid}>
              <Example title="Variants" wide>
                <div className={styles.surfaceGrid}>
                  <Panel className={styles.panelPreview}>
                    <h3>Surface</h3>
                    <p>Default grouped content.</p>
                  </Panel>
                  <Panel className={styles.panelPreview} variant="raised">
                    <h3>Raised</h3>
                    <p>Menus, dialogs, and focus surfaces.</p>
                  </Panel>
                  <Panel className={styles.panelPreview} variant="danger">
                    <h3>Danger</h3>
                    <p>Destructive settings and warning regions.</p>
                  </Panel>
                </div>
              </Example>

              <Example title="Padding" wide>
                <div className={styles.surfaceGrid}>
                  <Panel className={styles.panelPreview} padding="none">
                    <h3>None</h3>
                    <p>Fully custom inner layout.</p>
                  </Panel>
                  <Panel className={styles.panelPreview} padding="sm">
                    <h3>Small</h3>
                    <p>Dense metadata.</p>
                  </Panel>
                  <Panel className={styles.panelPreview} padding="md">
                    <h3>Medium</h3>
                    <p>Default panel spacing.</p>
                  </Panel>
                  <Panel className={styles.panelPreview} padding="lg">
                    <h3>Large</h3>
                    <p>Settings and detail surfaces.</p>
                  </Panel>
                </div>
              </Example>
            </div>
          </Section>

          <Section id="dialogs" title="Dialogs" description="Dialogs centralize modal structure, focus handling, close behavior, and action placement.">
            <div className={styles.exampleGrid}>
              <Example title="Modal Dialog">
                <Button icon={<IconPlus aria-hidden="true" />} onClick={() => setDialogOpen(true)} variant="primary">
                  Open dialog
                </Button>
              </Example>
            </div>
          </Section>

          <Section hideHeader id="popups" title="Popups">
            <div className={styles.exampleGrid}>
              <Example title="Playground" wide>
                <div className={styles.popupDemo}>
                  <div className={styles.popupControls}>
                    <Field label="Vertical alignment">
                      <SelectField
                        value={popupVertical}
                        onChange={(event) => {
                          setPopupVertical(event.target.value as PopupVertical);
                          setPopupOpen(true);
                        }}
                      >
                        {popupVerticalOptions.map((item) => (
                          <option key={item.value} value={item.value}>
                            {item.label}
                          </option>
                        ))}
                      </SelectField>
                    </Field>

                    <Field label="Horizontal alignment">
                      <SelectField
                        value={popupHorizontal}
                        onChange={(event) => {
                          setPopupHorizontal(event.target.value as PopupHorizontal);
                          setPopupOpen(true);
                        }}
                      >
                        {popupHorizontalOptions.map((item) => (
                          <option key={item.value} value={item.value}>
                            {item.label}
                          </option>
                        ))}
                      </SelectField>
                    </Field>

                    <Field label="Content">
                      <SelectField
                        value={popupContent}
                        onChange={(event) => {
                          setPopupContent(event.target.value as PopupContent);
                          setPopupOpen(true);
                        }}
                      >
                        {popupContentOptions.map((item) => (
                          <option key={item.value} value={item.value}>
                            {item.label}
                          </option>
                        ))}
                      </SelectField>
                    </Field>
                  </div>

                  <div className={styles.popupCell}>
                    <div className={styles.popupAnchor}>
                      <Button
                        aria-controls="catalog-popup-demo"
                        aria-expanded={popupOpen}
                        icon={<IconChevronDown aria-hidden="true" />}
                        iconPosition="end"
                        onClick={() => setPopupOpen((open) => !open)}
                      >
                        Activity
                      </Button>
                      {popupOpen ? (
                        <Popup id="catalog-popup-demo" placement={popupPlacement}>
                          <PopupPreviewContent content={popupContent} />
                        </Popup>
                      ) : null}
                    </div>
                  </div>
                </div>
              </Example>
            </div>
          </Section>
        </div>
      </div>
      {catalogNavOpen ? (
        <div className={styles.navOverlay}>
          <PressableButton
            aria-label="Close catalog navigation"
            className={styles.navBackdrop}
            onClick={() => setCatalogNavOpen(false)}
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
              <IconButton label="Close catalog navigation" onClick={() => setCatalogNavOpen(false)} size="sm">
                <IconX aria-hidden="true" />
              </IconButton>
            </div>
            <nav>
              <CatalogNavLinks onNavigate={() => setCatalogNavOpen(false)} />
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
