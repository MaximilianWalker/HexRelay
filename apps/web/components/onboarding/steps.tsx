import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

export type Step = "identity" | "recovery" | "access";

export function Steps({
  activeStep,
  finalLabel = "3. Access",
}: {
  activeStep: Step;
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
