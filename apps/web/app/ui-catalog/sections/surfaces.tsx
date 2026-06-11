import {
  IconCheck,
  IconChevronDown,
  IconLayoutGrid,
  IconList,
  IconMessageCircle,
  IconPinned,
  IconSettings,
  IconUserPlus,
} from "@tabler/icons-react";

import { Button } from "@/components/ui/buttons/button";
import { IconButton } from "@/components/ui/buttons/icon-button";
import { TextInput } from "@/components/ui/forms/text-input";
import { Panel } from "@/components/ui/surfaces/panel";
import { Toolbar } from "@/components/ui/surfaces/toolbar";
import { ToggleButton } from "@/components/ui/toggles/toggle-button";
import { ToggleGroup } from "@/components/ui/toggles/toggle-group";

import type { Filter, SectionId, ToggleGroupState } from "../data";
import { Example } from "../example";
import { Section } from "../section";

import styles from "../styles.module.css";

export function SurfacesSections({
  filter,
  isVisible,
  onFilterChange,
  onToggleGroupChange,
  toggleGroup,
}: {
  filter: Filter;
  isVisible: (sectionId: SectionId) => boolean;
  onFilterChange: (filter: Filter) => void;
  onToggleGroupChange: (value: ToggleGroupState) => void;
  toggleGroup: ToggleGroupState;
}) {
  return (
    <>
      <Section
        id="panels"
        title="Panels"
        visible={isVisible("panels")}
        description="Panels carry surface tone and padding variants without redefining controls."
      >
        <div className={styles.exampleGrid}>
          <Example title="Variants" wide>
            <div className={styles.surfaceGrid}>
              <Panel>
                <div className={styles.panelPreview}>
                  <h3>Surface</h3>
                  <p>Default grouped content.</p>
                </div>
              </Panel>
              <Panel variant="raised">
                <div className={styles.panelPreview}>
                  <h3>Raised</h3>
                  <p>Lists, dialogs, and focus surfaces.</p>
                </div>
              </Panel>
              <Panel variant="danger">
                <div className={styles.panelPreview}>
                  <h3>Danger</h3>
                  <p>Destructive settings and warning regions.</p>
                </div>
              </Panel>
            </div>
          </Example>

          <Example title="Padding" wide>
            <div className={styles.surfaceGrid}>
              <Panel padding="none">
                <div className={styles.panelPreview}>
                  <h3>None</h3>
                  <p>Fully custom inner layout.</p>
                </div>
              </Panel>
              <Panel padding="sm">
                <div className={styles.panelPreview}>
                  <h3>Small</h3>
                  <p>Dense metadata.</p>
                </div>
              </Panel>
              <Panel padding="md">
                <div className={styles.panelPreview}>
                  <h3>Medium</h3>
                  <p>Default panel spacing.</p>
                </div>
              </Panel>
              <Panel padding="lg">
                <div className={styles.panelPreview}>
                  <h3>Large</h3>
                  <p>Settings and detail surfaces.</p>
                </div>
              </Panel>
            </div>
          </Example>
        </div>
      </Section>

      <Section
        id="toolbar"
        title="Toolbar"
        visible={isVisible("toolbar")}
        description="Toolbars group repeated filters, view controls, search, and secondary actions without route-local control styling."
      >
        <div className={styles.exampleGrid}>
          <Example title="Filters" wide>
            <Toolbar
              actions={
                <>
                  <Button icon={<IconUserPlus aria-hidden="true" />} variant="primary">
                    Invite
                  </Button>
                  <IconButton label="Toolbar settings">
                    <IconSettings aria-hidden="true" />
                  </IconButton>
                </>
              }
            >
              <ToggleButton
                icon={<IconPinned aria-hidden="true" />}
                onPressedChange={() => onFilterChange(filter === "all" ? "muted" : "all")}
                pressed={filter === "all"}
              >
                Pinned
              </ToggleButton>
              <ToggleButton
                icon={<IconMessageCircle aria-hidden="true" />}
                onPressedChange={() => onFilterChange(filter === "unread" ? "all" : "unread")}
                pressed={filter === "unread"}
              >
                Unread
              </ToggleButton>
              <ToggleGroup
                label="Toolbar view mode"
                onChange={onToggleGroupChange}
                options={[
                  { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
                  { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
                ]}
                value={toggleGroup}
              />
            </Toolbar>
          </Example>

          <Example title="Search" wide>
            <Toolbar
              actions={
                <Button icon={<IconCheck aria-hidden="true" />} variant="primary">
                  Apply
                </Button>
              }
            >
              <TextInput aria-label="Toolbar search" placeholder="Search channels" />
              <Button icon={<IconChevronDown aria-hidden="true" />} iconPosition="end">
                Sort
              </Button>
            </Toolbar>
          </Example>
        </div>
      </Section>
    </>
  );
}
