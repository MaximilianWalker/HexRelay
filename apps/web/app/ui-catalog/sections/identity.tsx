import { BrandLockup } from "@/components/brand-lockup";
import { BrandLogo } from "@/components/brand-logo";

import type { SectionId } from "../data";
import { Example } from "../example";
import { Section } from "../section";

import styles from "../styles.module.css";

const logoSizes = [
  { className: styles.logoMarkSm, label: "Small", size: "sm" },
  { className: styles.logoMarkMd, label: "Medium", size: "md" },
  { className: styles.logoMarkLg, label: "Large", size: "lg" },
] as const;

export function IdentitySections({ isVisible }: { isVisible: (sectionId: SectionId) => boolean }) {
  return (
    <Section
      id="logo"
      title="Logo"
      visible={isVisible("logo")}
      description="The HexRelay mark and lockup scale together across compact, default, and large placements."
    >
      <div className={styles.exampleGrid}>
        <Example title="Logo Mark" wide>
          <div className={styles.logoSamples}>
            {logoSizes.map((logoSize) => (
              <div className={styles.logoSample} key={logoSize.size}>
                <span className={styles.sampleLabel}>{logoSize.label}</span>
                <BrandLogo
                  aria-label={`HexRelay logo ${logoSize.label.toLowerCase()}`}
                  className={`${styles.logoMark} ${logoSize.className}`}
                />
              </div>
            ))}
          </div>
        </Example>

        <Example title="Logo Lockup" wide>
          <div className={styles.logoSamples}>
            {logoSizes.map((logoSize) => (
              <div className={styles.logoSample} key={logoSize.size}>
                <span className={styles.sampleLabel}>{logoSize.label}</span>
                <BrandLockup size={logoSize.size} />
              </div>
            ))}
          </div>
        </Example>
      </div>
    </Section>
  );
}
