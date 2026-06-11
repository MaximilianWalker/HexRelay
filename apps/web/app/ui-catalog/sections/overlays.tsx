import { IconBellOff, IconChevronDown, IconCircleCheck, IconPlus, IconSettings } from "@tabler/icons-react";

import { Button } from "@/components/ui/buttons/button";
import { Alert } from "@/components/ui/feedback/alert";
import { Field } from "@/components/ui/forms/field";
import { SelectField } from "@/components/ui/forms/select-field";
import { List, ListButton } from "@/components/ui/navigation/list";
import { Popup, type PopupPlacement } from "@/components/ui/overlays/popup";
import { Panel } from "@/components/ui/surfaces/panel";

import type { SectionId } from "../data";
import { Example } from "../example";
import { Section } from "../section";

import styles from "../styles.module.css";

export type PopupContent = "alert" | "list" | "panel";
export type PopupHorizontal = "center" | "left" | "right";
export type PopupVertical = "bottom" | "center" | "top";

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
  { label: "List", value: "list" },
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

function PopupPreviewContent({ content }: { content: PopupContent }) {
  if (content === "list") {
    return (
      <List role="group">
        <ListButton icon={<IconSettings aria-hidden="true" />} name="Open settings" />
        <ListButton icon={<IconBellOff aria-hidden="true" />} name="Mute activity" />
      </List>
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

export function OverlaysSections({
  isVisible,
  onDialogOpenChange,
  onPopupContentChange,
  onPopupHorizontalChange,
  onPopupOpenChange,
  onPopupVerticalChange,
  popupContent,
  popupHorizontal,
  popupOpen,
  popupVertical,
}: {
  isVisible: (sectionId: SectionId) => boolean;
  onDialogOpenChange: (open: boolean) => void;
  onPopupContentChange: (content: PopupContent) => void;
  onPopupHorizontalChange: (horizontal: PopupHorizontal) => void;
  onPopupOpenChange: (open: boolean) => void;
  onPopupVerticalChange: (vertical: PopupVertical) => void;
  popupContent: PopupContent;
  popupHorizontal: PopupHorizontal;
  popupOpen: boolean;
  popupVertical: PopupVertical;
}) {
  const popupPlacement = getPopupPlacement(popupVertical, popupHorizontal);

  return (
    <>
      <Section
        id="dialogs"
        title="Dialogs"
        visible={isVisible("dialogs")}
        description="Dialogs centralize modal structure, focus handling, close behavior, and action placement."
      >
        <div className={styles.exampleGrid}>
          <Example title="Modal Dialog">
            <Button icon={<IconPlus aria-hidden="true" />} onClick={() => onDialogOpenChange(true)} variant="primary">
              Open dialog
            </Button>
          </Example>
        </div>
      </Section>

      <Section hideHeader id="popups" title="Popups" visible={isVisible("popups")}>
        <div className={styles.exampleGrid}>
          <Example title="Playground" wide>
            <div className={styles.popupDemo}>
              <div className={styles.popupControls}>
                <Field label="Vertical alignment">
                  <SelectField
                    value={popupVertical}
                    onChange={(event) => {
                      onPopupVerticalChange(event.target.value as PopupVertical);
                      onPopupOpenChange(true);
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
                      onPopupHorizontalChange(event.target.value as PopupHorizontal);
                      onPopupOpenChange(true);
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
                      onPopupContentChange(event.target.value as PopupContent);
                      onPopupOpenChange(true);
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
                    onClick={() => onPopupOpenChange(!popupOpen)}
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
    </>
  );
}
