import { IconCircleCheck } from "@tabler/icons-react";

import { Avatar } from "@/components/ui/display/avatar";
import { Badge } from "@/components/ui/display/badge";

import type { SectionId } from "../data";
import { Example } from "../example";
import { Section } from "../section";

import styles from "../styles.module.css";

export function DataDisplaySections({ isVisible }: { isVisible: (sectionId: SectionId) => boolean }) {
  return (
    <>
      <Section
        id="avatars"
        title="Avatars"
        visible={isVisible("avatars")}
        description="Avatars expose user and server shapes with the same three supported sizes."
      >
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

      <Section
        id="badges"
        title="Badges"
        visible={isVisible("badges")}
        description="Badges expose semantic tones and optional icons without changing text metrics."
      >
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
    </>
  );
}
