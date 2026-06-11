import { IconAlertTriangle, IconCircleCheck, IconCircleX, IconInfoCircle, IconPlus } from "@tabler/icons-react";

import { Button } from "@/components/ui/buttons/button";
import { Alert } from "@/components/ui/feedback/alert";
import { EmptyState } from "@/components/ui/feedback/empty-state";

import type { SectionId } from "../data";
import { Example } from "../example";
import { Section } from "../section";

import styles from "../styles.module.css";

export function FeedbackSections({ isVisible }: { isVisible: (sectionId: SectionId) => boolean }) {
  return (
    <>
      <Section
        id="alerts"
        title="Alerts"
        visible={isVisible("alerts")}
        description="Alerts expose every semantic tone through the same layout and typography."
      >
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

      <Section
        id="empty-states"
        title="Empty States"
        visible={isVisible("empty-states")}
        description="Empty states centralize title, copy, and optional action placement."
      >
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
    </>
  );
}
