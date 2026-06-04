"use client";

import { useEffect, useState, type ReactNode } from "react";
import {
  IconAlertTriangle,
  IconBell,
  IconBellOff,
  IconCheck,
  IconChevronDown,
  IconCircleCheck,
  IconHash,
  IconInfoCircle,
  IconLayoutGrid,
  IconList,
  IconMessageCircle,
  IconMicrophone,
  IconPinned,
  IconPlus,
  IconSearch,
  IconSend,
  IconServer2,
  IconSettings,
  IconTrash,
  IconUserPlus,
  IconVolume,
  IconX,
} from "@tabler/icons-react";

import { Avatar } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button, ButtonLink } from "@/components/ui/button";
import { CheckboxField } from "@/components/ui/checkbox-field";
import { EmptyState } from "@/components/ui/empty-state";
import { Field } from "@/components/ui/field";
import { IconButton } from "@/components/ui/icon-button";
import { ListActionButton } from "@/components/ui/list-action-button";
import { Menu, MenuItem } from "@/components/ui/menu";
import { Notice } from "@/components/ui/notice";
import { Panel } from "@/components/ui/panel";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { SelectField } from "@/components/ui/select-field";
import { TextArea } from "@/components/ui/text-area";
import { TextInput } from "@/components/ui/text-input";
import { ToggleButton } from "@/components/ui/toggle-button";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { Toolbar } from "@/components/ui/toolbar";

import styles from "./styles.module.css";

type Layout = "list" | "cards";
type Density = "comfortable" | "compact";
type Filter = "all" | "unread" | "muted";

const sections = [
  { id: "buttons", label: "Buttons" },
  { id: "toggles", label: "Toggles" },
  { id: "menus", label: "Menus" },
  { id: "lists", label: "List actions" },
  { id: "forms", label: "Forms" },
  { id: "feedback", label: "Feedback" },
  { id: "surfaces", label: "Surfaces" },
] as const;

function Section({
  children,
  description,
  id,
  title,
}: {
  children: ReactNode;
  description: string;
  id: string;
  title: string;
}) {
  return (
    <section className={styles.section} id={id}>
      <div className={styles.sectionHeader}>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
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

export function Demo() {
  const [compact, setCompact] = useState(false);
  const [density, setDensity] = useState<Density>("comfortable");
  const [filter, setFilter] = useState<Filter>("muted");
  const [layout, setLayout] = useState<Layout>("cards");
  const [muted, setMuted] = useState(true);
  const [pinned, setPinned] = useState(true);

  useEffect(() => {
    function scrollToHash(): void {
      const id = window.location.hash.slice(1);
      if (!id) {
        return;
      }

      window.requestAnimationFrame(() => {
        document.getElementById(id)?.scrollIntoView({ block: "start" });
      });
    }

    scrollToHash();
    window.addEventListener("hashchange", scrollToHash);

    return () => window.removeEventListener("hashchange", scrollToHash);
  }, []);

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div>
          <p className={styles.eyebrow}>Development catalog</p>
          <h1>HexRelay UI framework</h1>
          <p className={styles.summary}>
            Shared primitives, states, tones, and composed patterns used by app surfaces.
          </p>
        </div>
        <Badge tone="accent" icon={<IconCircleCheck aria-hidden="true" />}>
          Shared APIs
        </Badge>
      </header>

      <div className={styles.shell}>
        <aside className={styles.nav} aria-label="UI catalog sections">
          <p className={styles.navTitle}>Catalog</p>
          <nav>
            {sections.map((section) => (
              <a href={`#${section.id}`} key={section.id}>
                {section.label}
              </a>
            ))}
          </nav>
        </aside>

        <div className={styles.content}>
          <Section
            id="buttons"
            title="Buttons"
            description="One shared Button contract covers text, icon-text, icon-only, link, active, disabled, and loading states."
          >
            <div className={styles.exampleGrid}>
              <Example title="Variants">
                <div className={styles.row}>
                  <Button variant="primary" icon={<IconPlus aria-hidden="true" />}>
                    Create
                  </Button>
                  <Button variant="secondary" icon={<IconSettings aria-hidden="true" />}>
                    Settings
                  </Button>
                  <Button variant="ghost" icon={<IconBell aria-hidden="true" />}>
                    Notify
                  </Button>
                  <Button variant="danger" icon={<IconTrash aria-hidden="true" />}>
                    Delete
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
                  <ToggleButton icon={<IconPinned aria-hidden="true" />} onPressedChange={setPinned} pressed={pinned}>
                    Pinned
                  </ToggleButton>
                  <Button disabled icon={<IconX aria-hidden="true" />}>
                    Disabled
                  </Button>
                  <Button loading icon={<IconSend aria-hidden="true" />} variant="primary">
                    Sending
                  </Button>
                  <ButtonLink href="/servers" icon={<IconServer2 aria-hidden="true" />} variant="secondary">
                    Server link
                  </ButtonLink>
                </div>
              </Example>

              <Example title="Small size">
                <div className={styles.row}>
                  <Button size="sm">Compact</Button>
                  <Button icon={<IconCheck aria-hidden="true" />} size="sm">
                    Accepted
                  </Button>
                  <Button icon={<IconTrash aria-hidden="true" />} size="sm" variant="danger">
                    Remove
                  </Button>
                </div>
              </Example>
            </div>
          </Section>

          <Section
            id="toggles"
            title="Toggles"
            description="Toggle buttons and segmented controls share active color, font, weight, and icon sizing."
          >
            <div className={styles.exampleGrid}>
              <Example title="Filter buttons">
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

              <Example title="Segmented controls">
                <div className={styles.column}>
                  <SegmentedControl
                    label="Catalog layout"
                    onChange={setLayout}
                    options={[
                      { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
                      { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
                    ]}
                    value={layout}
                  />
                  <SegmentedControl
                    label="Catalog density"
                    onChange={setDensity}
                    options={[
                      { id: "comfortable", label: "Comfortable" },
                      { id: "compact", label: "Compact" },
                    ]}
                    value={density}
                  />
                </div>
              </Example>

              <Example title="Switches">
                <div className={styles.row}>
                  <span className={styles.switchLabel}>Sound</span>
                  <ToggleSwitch checked={!muted} label="Sound enabled" onChange={(next) => setMuted(!next)} />
                  <span className={styles.switchLabel}>Compact mode</span>
                  <ToggleSwitch checked={compact} label="Compact mode" onChange={setCompact} />
                  <span className={styles.switchLabel}>Disabled</span>
                  <ToggleSwitch checked={false} disabled label="Disabled switch" />
                </div>
              </Example>
            </div>
          </Section>

          <Section
            id="menus"
            title="Menus"
            description="Menus centralize item layout, keyboard focus movement, icon slots, danger tone, and checked state."
          >
            <div className={styles.exampleGrid}>
              <Example title="Profile menu">
                <Menu className={styles.menu} position="static">
                  <MenuItem icon={<IconSettings aria-hidden="true" />}>Profile settings</MenuItem>
                  <MenuItem icon={<IconBellOff aria-hidden="true" />} pressed={muted}>
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

              <Example title="Compact actions">
                <Menu className={styles.menu} position="static" role="group">
                  <MenuItem icon={<IconPinned aria-hidden="true" />} role="button">
                    Pin tab
                  </MenuItem>
                  <MenuItem disabled icon={<IconVolume aria-hidden="true" />} role="button">
                    Voice unavailable
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
              <Example title="Channels">
                <div className={styles.stack}>
                  <ListActionButton active badge="8" badgeLabel="8 unread" icon={<IconHash aria-hidden="true" />}>
                    general
                  </ListActionButton>
                  <ListActionButton icon={<IconMicrophone aria-hidden="true" />}>Lobby</ListActionButton>
                  <ListActionButton disabled icon={<IconBellOff aria-hidden="true" />}>
                    muted-updates
                  </ListActionButton>
                </div>
              </Example>

              <Example title="Toolbar recipe">
                <Toolbar
                  actions={
                    <>
                      <Button icon={<IconPlus aria-hidden="true" />} variant="primary">
                        Add
                      </Button>
                      <IconButton label="More actions">
                        <IconChevronDown aria-hidden="true" />
                      </IconButton>
                    </>
                  }
                  className={styles.toolbarExample}
                >
                  <TextInput aria-label="Search catalog entries" placeholder="Search" />
                </Toolbar>
              </Example>
            </div>
          </Section>

          <Section id="forms" title="Forms" description="Fields share label, helper, invalid, disabled, and control typography styles.">
            <div className={styles.exampleGrid}>
              <Example title="Inputs" wide>
                <div className={styles.fieldGrid}>
                  <Field helper="Shown in profile cards and mentions." label="Display name">
                    <TextInput defaultValue="Diogo" />
                  </Field>
                  <Field label="Status" helper="Visible to contacts.">
                    <SelectField defaultValue="online">
                      <option value="online">Online</option>
                      <option value="away">Away</option>
                      <option value="offline">Offline</option>
                    </SelectField>
                  </Field>
                  <Field error="Server name is required." label="Server name">
                    <TextInput invalid placeholder="Product team" />
                  </Field>
                  <Field label="Welcome note">
                    <TextArea defaultValue="Share release notes, planning threads, and voice rooms here." rows={4} />
                  </Field>
                </div>
              </Example>

              <Example title="Checks">
                <div className={styles.column}>
                  <CheckboxField defaultChecked>Allow contact requests</CheckboxField>
                  <CheckboxField>Send desktop notifications</CheckboxField>
                  <CheckboxField disabled>Locked by server role</CheckboxField>
                </div>
              </Example>
            </div>
          </Section>

          <Section id="feedback" title="Feedback" description="Badges, notices, avatars, and empty states use semantic tones only.">
            <div className={styles.exampleGrid}>
              <Example title="Badges and avatars">
                <div className={styles.column}>
                  <div className={styles.row}>
                    <Badge>Neutral</Badge>
                    <Badge tone="accent">Accent</Badge>
                    <Badge tone="success">Synced</Badge>
                    <Badge tone="warning">Pending</Badge>
                    <Badge tone="danger">Blocked</Badge>
                  </div>
                  <div className={styles.row}>
                    <Avatar label="Diogo" size="sm" text="DG" />
                    <Avatar label="HexRelay" kind="server" text="HR" />
                    <Avatar label="Design system" kind="server" size="lg" text="UI" />
                  </div>
                </div>
              </Example>

              <Example title="Notices">
                <div className={styles.stack}>
                  <Notice icon={<IconInfoCircle aria-hidden="true" />}>Catalog examples run only in development.</Notice>
                  <Notice icon={<IconCircleCheck aria-hidden="true" />} tone="success">
                    Shared control styles are validated by lint.
                  </Notice>
                  <Notice icon={<IconAlertTriangle aria-hidden="true" />} tone="warning">
                    Route-local copies should be migrated before reuse.
                  </Notice>
                </div>
              </Example>

              <Example title="Empty state">
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

          <Section id="surfaces" title="Surfaces" description="Panels carry surface tone and padding variants without redefining controls.">
            <div className={styles.surfaceGrid}>
              <Panel className={styles.panelPreview} padding="sm">
                <h3>Small padding</h3>
                <p>Dense metadata or compact settings.</p>
              </Panel>
              <Panel className={styles.panelPreview} padding="md" variant="raised">
                <h3>Raised panel</h3>
                <p>Menus, dialogs, and focus surfaces.</p>
              </Panel>
              <Panel className={styles.panelPreview} padding="lg" variant="danger">
                <h3>Danger panel</h3>
                <p>Destructive settings and warning regions.</p>
              </Panel>
            </div>
          </Section>
        </div>
      </div>
    </main>
  );
}
