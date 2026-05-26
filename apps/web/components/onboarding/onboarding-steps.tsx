import { cx } from "@/lib/ui/cx";

import styles from "./onboarding.module.css";

export type OnboardingStep = "identity" | "recovery" | "access";

export function OnboardingSteps({
  activeStep,
  finalLabel = "3. Access",
}: {
  activeStep: OnboardingStep;
  finalLabel?: string;
}) {
  const steps = [
    { id: "identity", label: "1. Identity" },
    { id: "recovery", label: "2. Recovery" },
    { id: "access", label: finalLabel },
  ] as const;

  return (
    <div className={styles.steps}>
      {steps.map((step) => (
        <div className={cx(styles.step, activeStep === step.id && styles.activeStep)} key={step.id}>
          {step.label}
        </div>
      ))}
    </div>
  );
}
