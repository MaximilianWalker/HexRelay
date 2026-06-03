"use client";

import { Shell } from "@/components/onboarding/shell";
import { ButtonLink } from "@/components/ui/button";
import { Notice } from "@/components/ui/notice";
import { Panel } from "@/components/ui/panel";
import styles from "../styles.module.css";

export default function AccessOnboardingPage() {
  return (
    <Shell
      activeStep="access"
      finalStepLabel="3. Complete"
      introBody="Server join and friend-request actions now happen inside the main app to keep onboarding focused and predictable."
      introTitle="You are ready to enter HexRelay"
      promises={[
        "Onboarding stays focused on identity and recovery only.",
        "Join server and add contact flows are in app surfaces.",
        "You can start testing immediately from Home and Hubs.",
      ]}
      statusItems={["Identity: ready", "Recovery: confirmed", "Access flows: moved to app hubs"]}
      wizardSubtitle="Continue to the app. Use Servers and Contacts hubs for join and request workflows."
      wizardTitle="Onboarding complete"
    >
      <Notice className={styles.notice} tone="success">
        Identity and recovery checkpoints passed.
      </Notice>

      <div className={styles.choiceGrid}>
        <Panel className={styles.choice} padding="sm">
          <p className={styles.choiceTitle}>Join server in-app</p>
          <p className={styles.choiceText}>Use the Servers hub after onboarding.</p>
        </Panel>
        <Panel className={styles.choice} padding="sm">
          <p className={styles.choiceTitle}>Manage contacts in-app</p>
          <p className={styles.choiceText}>Use the Contacts hub for friend requests.</p>
        </Panel>
      </div>

      <div className={styles.ctaRow}>
        <ButtonLink href="/onboarding/recovery" variant="ghost">
          Back to recovery
        </ButtonLink>
        <ButtonLink href="/home" variant="primary">
          Enter HexRelay
        </ButtonLink>
      </div>
    </Shell>
  );
}
