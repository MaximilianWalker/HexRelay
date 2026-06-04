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
  IconMessageCircle,
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
import { ButtonGroup } from "@/components/ui/button-group";
import { Button, ButtonLink } from "@/components/ui/button";
import { CheckboxField } from "@/components/ui/checkbox-field";
import { Dialog } from "@/components/ui/dialog";
import { DialogActions } from "@/components/ui/dialog-actions";
import { EmptyState } from "@/components/ui/empty-state";
import { Field } from "@/components/ui/field";
import { IconButton } from "@/components/ui/icon-button";
import { ListActionButton } from "@/components/ui/list-action-button";
import { Menu, MenuItem } from "@/components/ui/menu";
import { Alert } from "@/components/ui/alert";
import { Panel } from "@/components/ui/panel";
import { SelectField } from "@/components/ui/select-field";
import { TextArea } from "@/components/ui/text-area";
import { TextInput } from "@/components/ui/text-input";
import { ToggleButton } from "@/components/ui/toggle-button";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { Toolbar } from "@/components/ui/toolbar";

import styles from "./styles.module.css";

type ButtonGroupState = "list" | "cards" | "disabled";
type Filter = "all" | "unread" | "muted";

const sections = [
  { id: "buttons", label: "Buttons" },
  { id: "toggles", label: "Toggles" },
  { id: "menus", label: "Menus" },
  { id: "lists", label: "List Actions" },
  { id: "toolbars", label: "Toolbars" },
  { id: "forms", label: "Forms" },
  { id: "badges", label: "Badges" },
  { id: "avatars", label: "Avatars" },
  { id: "alerts", label: "Alerts" },
  { id: "empty-states", label: "Empty States" },
  { id: "panels", label: "Panels" },
  { id: "dialogs", label: "Dialogs" },
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
  const [buttonGroup, setButtonGroup] = useState<ButtonGroupState>("list");
  const [filter, setFilter] = useState<Filter>("muted");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [switchOff, setSwitchOff] = useState(false);
  const [switchOn, setSwitchOn] = useState(true);
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
                      Pinned
                    </ToggleButton>
                    <Button disabled icon={<IconX aria-hidden="true" />}>
                      Disabled
                    </Button>
                    <Button loading icon={<IconSend aria-hidden="true" />} variant="primary">
                      Loading
                    </Button>
                    <ButtonLink href="/servers" icon={<IconServer2 aria-hidden="true" />} variant="secondary">
                      Link
                    </ButtonLink>
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
            <div className={styles.exampleGrid}>
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

              <Example title="Button Groups">
                <ButtonGroup
                  label="View mode"
                  onChange={setButtonGroup}
                  options={[
                    { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
                    { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
                    { id: "disabled", label: "Disabled", disabled: true },
                  ]}
                  value={buttonGroup}
                />
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
          </Section>

          <Section
            id="menus"
            title="Menus"
            description="Menus centralize item layout, keyboard focus movement, icon slots, danger tone, and checked state."
          >
            <div className={styles.exampleGrid}>
              <Example title="Menu Items">
                <Menu className={styles.menu} position="static">
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

              <Example title="Action Menu">
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
            </div>
          </Section>

          <Section
            id="toolbars"
            title="Toolbars"
            description="Toolbars group primary filters and secondary actions without redefining button layout."
          >
            <div className={styles.exampleGrid}>
              <Example title="Search Toolbar" wide>
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

              <Example title="Action Toolbar">
                <Toolbar
                  actions={
                    <>
                      <Button icon={<IconCheck aria-hidden="true" />}>Accept</Button>
                      <Button icon={<IconTrash aria-hidden="true" />} variant="danger">
                        Remove
                      </Button>
                    </>
                  }
                  className={styles.toolbarExample}
                >
                  <Button icon={<IconPinned aria-hidden="true" />} pressed>
                    Pinned
                  </Button>
                  <Button icon={<IconBell aria-hidden="true" />}>Notify</Button>
                </Toolbar>
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
                    <Badge size="sm" tone="accent">
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
                    <Badge tone="accent">12</Badge>
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
                    <Badge size="lg" tone="accent">
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
        </div>
      </div>
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
