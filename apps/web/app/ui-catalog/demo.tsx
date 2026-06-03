"use client";

import { useState } from "react";
import { IconBellOff, IconLayoutGrid, IconList, IconMessageCircle, IconPinned, IconVolume } from "@tabler/icons-react";

import { Badge } from "@/components/ui/badge";
import { Button, ButtonLink } from "@/components/ui/button";
import { ListActionButton } from "@/components/ui/list-action-button";
import { Menu, MenuItem } from "@/components/ui/menu";
import { Panel } from "@/components/ui/panel";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { ToggleButton } from "@/components/ui/toggle-button";
import { ToggleSwitch } from "@/components/ui/toggle-switch";

import styles from "./styles.module.css";

type Layout = "list" | "cards";

export function Demo() {
  const [layout, setLayout] = useState<Layout>("cards");
  const [muted, setMuted] = useState(true);
  const [compact, setCompact] = useState(false);

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div>
          <p className={styles.eyebrow}>Development catalog</p>
          <h1>HexRelay UI controls</h1>
        </div>
        <Badge tone="accent">Shared APIs</Badge>
      </header>

      <section className={styles.grid}>
        <Panel className={styles.section} padding="md">
          <h2>Buttons</h2>
          <div className={styles.row}>
            <Button icon={<IconPinned aria-hidden="true" />} pressed>
              Pinned
            </Button>
            <ToggleButton icon={<IconBellOff aria-hidden="true" />} onPressedChange={setMuted} pressed={muted}>
              Muted
            </ToggleButton>
            <ButtonLink href="/servers" variant="primary">
              Servers
            </ButtonLink>
            <Button disabled>Disabled</Button>
          </div>
        </Panel>

        <Panel className={styles.section} padding="md">
          <h2>Toggle Groups</h2>
          <div className={styles.row}>
            <SegmentedControl
              label="Catalog layout"
              onChange={setLayout}
              options={[
                { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
                { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
              ]}
              value={layout}
            />
            <ToggleSwitch checked={compact} label="Compact controls" onChange={setCompact} />
          </div>
        </Panel>

        <Panel className={styles.section} padding="md">
          <h2>Menus</h2>
          <Menu className={styles.menu} position="static">
            <MenuItem icon={<IconPinned aria-hidden="true" />}>Pin tab</MenuItem>
            <MenuItem icon={<IconVolume aria-hidden="true" />} pressed={muted}>
              Mute notifications
            </MenuItem>
            <MenuItem icon={<IconBellOff aria-hidden="true" />} tone="danger">
              Leave server
            </MenuItem>
          </Menu>
        </Panel>

        <Panel className={styles.section} padding="md">
          <h2>List Actions</h2>
          <div className={styles.stack}>
            <ListActionButton active badge="2" badgeLabel="2 unread" icon={<IconMessageCircle aria-hidden="true" />}>
              general
            </ListActionButton>
            <ListActionButton icon={<IconVolume aria-hidden="true" />}>Lobby</ListActionButton>
          </div>
        </Panel>
      </section>
    </main>
  );
}
