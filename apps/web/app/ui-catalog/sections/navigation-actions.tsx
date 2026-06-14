import {
  IconBell,
  IconBellOff,
  IconHash,
  IconInfoCircle,
  IconLayoutGrid,
  IconMessageCircle,
  IconPinned,
  IconServer2,
  IconSettings,
  IconTrash,
  IconUserPlus,
  IconVolume,
} from "@tabler/icons-react";

import { Badge } from "@/components/ui/display/badge";
import { List, ListButton, ListRow } from "@/components/ui/navigation/list";
import { Menu } from "@/components/ui/navigation/menu";
import { ScrollArea } from "@/components/ui/navigation/scroll-area";
import { ScrollButton } from "@/components/ui/navigation/scroll-button";

import type { SectionId } from "../data";
import { Example } from "../example";
import { Section } from "../section";

import styles from "../styles.module.css";

const scrollAreaItems = [
  "Announcements",
  "Design review",
  "Backend sync",
  "Release checklist",
  "Support triage",
  "Voice room notes",
  "Mobile polish",
  "Security follow-up",
] as const;

export function NavigationActionsSections({ isVisible }: { isVisible: (sectionId: SectionId) => boolean }) {
  return (
    <>
      <Section
        id="list"
        title="List"
        visible={isVisible("list")}
        description="Lists provide basic customizable rows with name, icon, end slot, state, size, and tone props."
      >
        <div className={styles.exampleGrid}>
          <Example title="Items">
            <List role="menu">
              <ListButton icon={<IconSettings aria-hidden="true" />} name="Profile settings" role="menuitem" />
              <ListButton
                icon={<IconBellOff aria-hidden="true" />}
                name="Mute notifications"
                pressed
                role="menuitemcheckbox"
              />
              <ListButton
                end={<Badge tone="muted">New</Badge>}
                icon={<IconUserPlus aria-hidden="true" />}
                name="Invite contact"
                role="menuitem"
              />
              <ListButton icon={<IconTrash aria-hidden="true" />} name="Leave server" role="menuitem" tone="danger" />
            </List>
          </Example>

          <Example title="Actions">
            <List role="group">
              <ListButton icon={<IconPinned aria-hidden="true" />} name="Pin tab" />
              <ListButton disabled icon={<IconVolume aria-hidden="true" />} name="Voice unavailable" />
            </List>
          </Example>

          <Example title="Static Content">
            <List role="group">
              <ListRow
                end={<Badge tone="muted">Current</Badge>}
                icon={<IconInfoCircle aria-hidden="true" />}
                name="Sidebar layout"
              />
              <ListButton icon={<IconSettings aria-hidden="true" />} name="Edit preferences" />
            </List>
          </Example>

          <Example title="Sizes">
            <List role="group">
              <ListButton icon={<IconSettings aria-hidden="true" />} name="Small" size="sm" />
              <ListButton icon={<IconPinned aria-hidden="true" />} name="Medium" />
              <ListButton icon={<IconVolume aria-hidden="true" />} name="Large" size="lg" />
            </List>
          </Example>

          <Example title="Without Panel">
            <List panel={false} role="group">
              <ListButton icon={<IconSettings aria-hidden="true" />} name="Plain action" />
              <ListButton icon={<IconBellOff aria-hidden="true" />} name="Muted row" />
              <ListRow
                end={<Badge tone="muted">Static</Badge>}
                icon={<IconInfoCircle aria-hidden="true" />}
                name="Plain row"
              />
            </List>
          </Example>
        </div>
      </Section>

      <Section
        id="menu"
        title="Menu"
        visible={isVisible("menu")}
        description="Menu composes lists from objects for sidebars, channel rails, actions, and nested navigation."
      >
        <div className={styles.exampleGrid}>
          <Example title="States">
            <Menu
              activeId="messages"
              items={[
                { icon: <IconHash aria-hidden="true" />, id: "default", name: "Default" },
                { icon: <IconMessageCircle aria-hidden="true" />, id: "messages", name: "Active" },
                {
                  end: (
                    <Badge aria-label="8 unread" shape="counter" size="sm" tone="accent">
                      8
                    </Badge>
                  ),
                  icon: <IconHash aria-hidden="true" />,
                  id: "badge",
                  name: "With badge",
                },
                { disabled: true, icon: <IconBellOff aria-hidden="true" />, id: "disabled", name: "Disabled" },
              ]}
              panel={false}
            />
          </Example>

          <Example title="Sizes">
            <Menu
              items={[
                { icon: <IconHash aria-hidden="true" />, id: "small", name: "Small", size: "sm" },
                { icon: <IconMessageCircle aria-hidden="true" />, id: "medium", name: "Medium" },
                { icon: <IconServer2 aria-hidden="true" />, id: "large", name: "Large", size: "lg" },
              ]}
              panel={false}
            />
          </Example>

          <Example title="Nested">
            <Menu
              activeId="forms"
              defaultExpandedIds={["controls"]}
              items={[
                {
                  icon: <IconLayoutGrid aria-hidden="true" />,
                  id: "controls",
                  items: [
                    { href: "#buttons", id: "buttons-link", name: "Buttons" },
                    { current: false, href: "#forms", id: "forms", name: "Forms" },
                  ],
                  name: "Inputs & Controls",
                },
                {
                  icon: <IconServer2 aria-hidden="true" />,
                  id: "workspace",
                  items: [
                    { id: "channels", name: "Channels" },
                    { id: "voice", name: "Voice" },
                  ],
                  name: "Workspace",
                },
              ]}
              panel={false}
            />
          </Example>

          <Example title="Panel Menu">
            <Menu
              activeId="mentions"
              items={[
                { icon: <IconMessageCircle aria-hidden="true" />, id: "mentions", name: "Mentions" },
                { icon: <IconHash aria-hidden="true" />, id: "channels", name: "Channels" },
                {
                  end: <Badge tone="muted">3</Badge>,
                  icon: <IconBell aria-hidden="true" />,
                  id: "alerts",
                  name: "Alerts",
                },
              ]}
              panel
            />
          </Example>

          <Example title="Compact Panel">
            <Menu
              activeId="servers"
              activeIndicator="rail"
              iconColor="accent"
              idleBorder={false}
              items={[
                { icon: <IconLayoutGrid aria-hidden="true" />, id: "home", name: "Home" },
                { icon: <IconServer2 aria-hidden="true" />, id: "servers", name: "Servers" },
                { icon: <IconSettings aria-hidden="true" />, id: "settings", name: "Settings" },
              ]}
              panel
              spacing="sm"
            />
          </Example>
        </div>
      </Section>

      <Section
        id="scroll-area"
        title="Scroll Area"
        visible={isVisible("scroll-area")}
        description="Scroll areas provide a consistent viewport and scrollbar treatment for dense component surfaces."
      >
        <div className={styles.exampleGrid}>
          <Example title="Overlay Scrollbar">
            <ScrollArea className={styles.scrollAreaDemo} hideWhenIdle width={6}>
              <div className={styles.scrollAreaContent}>
                {scrollAreaItems.map((item) => (
                  <p className={styles.scrollAreaItem} key={item}>
                    {item}
                  </p>
                ))}
              </div>
            </ScrollArea>
          </Example>

          <Example title="Reserved Track">
            <ScrollArea className={styles.scrollAreaDemo} overlay={false} width={8}>
              <div className={styles.scrollAreaContent}>
                {scrollAreaItems.map((item) => (
                  <p className={styles.scrollAreaItem} key={item}>
                    {item}
                  </p>
                ))}
              </div>
            </ScrollArea>
          </Example>

          <Example title="Scroll Buttons">
            <div className={styles.row}>
              <ScrollButton direction="previous" label="Previous item" />
              <ScrollButton appearance="plain" direction="next" label="Next item" />
            </div>
          </Example>
        </div>
      </Section>
    </>
  );
}
