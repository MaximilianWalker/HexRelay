import {
  IconBell,
  IconBellOff,
  IconCheck,
  IconChevronDown,
  IconCircleCheck,
  IconLayoutGrid,
  IconList,
  IconMessageCircle,
  IconPinned,
  IconPlus,
  IconSearch,
  IconServer2,
  IconSettings,
  IconTrash,
  IconX,
} from "@tabler/icons-react";

import { Button } from "@/components/ui/buttons/button";
import { IconButton } from "@/components/ui/buttons/icon-button";
import { CheckboxField } from "@/components/ui/forms/checkbox-field";
import { Field } from "@/components/ui/forms/field";
import { SelectField } from "@/components/ui/forms/select-field";
import { TextArea } from "@/components/ui/forms/text-area";
import { TextInput } from "@/components/ui/forms/text-input";
import { ToggleButton } from "@/components/ui/toggles/toggle-button";
import { ToggleGroup } from "@/components/ui/toggles/toggle-group";
import { ToggleSwitch } from "@/components/ui/toggles/toggle-switch";

import type { Filter, SectionId, ToggleGroupState } from "../data";
import { Example } from "../example";
import { Section } from "../section";

import styles from "../styles.module.css";

export function InputsControlsSections({
  filter,
  isVisible,
  onFilterChange,
  onPinnedChange,
  onSwitchOffChange,
  onSwitchOnChange,
  onToggleGroupChange,
  pinned,
  switchOff,
  switchOn,
  toggleGroup,
}: {
  filter: Filter;
  isVisible: (sectionId: SectionId) => boolean;
  onFilterChange: (filter: Filter) => void;
  onPinnedChange: (pinned: boolean) => void;
  onSwitchOffChange: (checked: boolean) => void;
  onSwitchOnChange: (checked: boolean) => void;
  onToggleGroupChange: (value: ToggleGroupState) => void;
  pinned: boolean;
  switchOff: boolean;
  switchOn: boolean;
  toggleGroup: ToggleGroupState;
}) {
  return (
    <>
      <Section
        id="buttons"
        title="Buttons"
        visible={isVisible("buttons")}
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
                <ToggleButton icon={<IconPinned aria-hidden="true" />} onPressedChange={onPinnedChange} pressed={pinned}>
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
        visible={isVisible("toggles")}
        description="Toggle buttons, toggle groups, and switches share active color, font, weight, and icon sizing."
      >
        <div className={styles.buttonLayout}>
          <div className={styles.buttonStack}>
            <Example title="Toggle Buttons">
              <div className={styles.row}>
                <ToggleButton
                  icon={<IconMessageCircle aria-hidden="true" />}
                  onPressedChange={() => onFilterChange(filter === "unread" ? "all" : "unread")}
                  pressed={filter === "unread"}
                >
                  Unread
                </ToggleButton>
                <ToggleButton
                  icon={<IconBellOff aria-hidden="true" />}
                  onPressedChange={() => onFilterChange(filter === "muted" ? "all" : "muted")}
                  pressed={filter === "muted"}
                >
                  Muted
                </ToggleButton>
                <ToggleButton
                  icon={<IconPinned aria-hidden="true" />}
                  onPressedChange={() => onFilterChange(filter === "all" ? "muted" : "all")}
                  pressed={filter === "all"}
                >
                  All
                </ToggleButton>
              </div>
            </Example>

            <Example title="Switches">
              <div className={styles.row}>
                <span className={styles.switchLabel}>Off</span>
                <ToggleSwitch checked={switchOff} label="Off switch" onChange={onSwitchOffChange} />
                <span className={styles.switchLabel}>On</span>
                <ToggleSwitch checked={switchOn} label="On switch" onChange={onSwitchOnChange} />
                <span className={styles.switchLabel}>Disabled</span>
                <ToggleSwitch checked={false} disabled label="Disabled switch" />
              </div>
            </Example>
          </div>

          <Example title="Sizes">
            <div className={styles.controlSizeList}>
              <span className={styles.matrixLabel}>Small</span>
              <ToggleGroup
                label="Small view mode"
                onChange={onToggleGroupChange}
                options={[
                  { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
                  { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
                  { id: "disabled", label: "Disabled", disabled: true },
                ]}
                size="sm"
                value={toggleGroup}
              />
              <span className={styles.matrixLabel}>Medium</span>
              <ToggleGroup
                label="Medium view mode"
                onChange={onToggleGroupChange}
                options={[
                  { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
                  { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
                  { id: "disabled", label: "Disabled", disabled: true },
                ]}
                value={toggleGroup}
              />
              <span className={styles.matrixLabel}>Large</span>
              <ToggleGroup
                label="Large view mode"
                onChange={onToggleGroupChange}
                options={[
                  { id: "list", label: "List", icon: <IconList aria-hidden="true" /> },
                  { id: "cards", label: "Cards", icon: <IconLayoutGrid aria-hidden="true" /> },
                  { id: "disabled", label: "Disabled", disabled: true },
                ]}
                size="lg"
                value={toggleGroup}
              />
            </div>
          </Example>
        </div>
      </Section>

      <Section
        id="forms"
        title="Forms"
        visible={isVisible("forms")}
        description="Fields share label, helper, invalid, disabled, and control typography styles."
      >
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
    </>
  );
}
