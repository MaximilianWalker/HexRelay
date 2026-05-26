import type { ReactNode } from "react";

import { OnboardingSteps, type OnboardingStep } from "./onboarding-steps";
import styles from "./onboarding.module.css";

export function OnboardingShell({
  activeStep,
  children,
  finalStepLabel,
  introBody,
  introTitle,
  promises,
  statusItems,
  wizardSubtitle,
  wizardTitle,
}: {
  activeStep: OnboardingStep;
  children: ReactNode;
  finalStepLabel?: string;
  introBody: ReactNode;
  introTitle: string;
  promises: string[];
  statusItems: string[];
  wizardSubtitle: ReactNode;
  wizardTitle: string;
}) {
  return (
    <div className={styles.shell}>
      <div className={styles.content}>
        <aside className={styles.panel}>
          <p className={styles.brandEyebrow}>HexRelay onboarding</p>
          <h1 className={styles.leftTitle}>{introTitle}</h1>
          <p className={styles.leftBody}>{introBody}</p>
          <ul className={styles.promiseList}>
            {promises.map((promise) => (
              <li key={promise}>{promise}</li>
            ))}
          </ul>
        </aside>

        <main className={styles.panel}>
          <OnboardingSteps activeStep={activeStep} finalLabel={finalStepLabel} />
          <h2 className={styles.wizardTitle}>{wizardTitle}</h2>
          <p className={styles.wizardSubtitle}>{wizardSubtitle}</p>
          {children}
        </main>

        <aside className={styles.panel}>
          <h3 className={styles.wizardTitle}>Setup status</h3>
          <div className={styles.asideList}>
            {statusItems.map((item) => (
              <div className={styles.asideItem} key={item}>
                {item}
              </div>
            ))}
          </div>
        </aside>
      </div>
    </div>
  );
}
