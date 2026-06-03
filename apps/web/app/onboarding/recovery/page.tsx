"use client";

import { useEffect, useMemo, useState } from "react";

import { ButtonLink } from "@/components/ui/button";
import { readActivePersonaId } from "@/lib/personas";
import { getOrCreateRecoveryPhraseForPersona } from "@/lib/recovery";
import { Shell } from "@/components/onboarding/shell";
import { Field } from "@/components/ui/field";
import { Notice } from "@/components/ui/notice";
import { TextInput } from "@/components/ui/text-input";
import styles from "../styles.module.css";

export default function RecoveryOnboardingPage() {
  const [phrase, setPhrase] = useState<string[]>([]);
  const [word3, setWord3] = useState("");
  const [word7, setWord7] = useState("");
  const [word11, setWord11] = useState("");

  useEffect(() => {
    const personaId = readActivePersonaId();
    if (!personaId) {
      return;
    }

    void getOrCreateRecoveryPhraseForPersona(personaId).then((resolved) => {
      setPhrase(resolved);
    });
  }, []);

  const confirmed = useMemo(
    () =>
      phrase.length === 12 &&
      word3.trim().toLowerCase() === phrase[2] &&
      word7.trim().toLowerCase() === phrase[6] &&
      word11.trim().toLowerCase() === phrase[10],
    [phrase, word11, word3, word7],
  );

  return (
    <Shell
      activeStep="recovery"
      introBody="Recovery confirmation is mandatory. Onboarding cannot finish without this step."
      introTitle="Confirm recovery phrase"
      promises={[
        "Phrase is displayed once for backup.",
        "Never send this phrase through chat channels.",
        "Losing phrase means no key recovery.",
      ]}
      statusItems={["Identity: ready", `Recovery: ${confirmed ? "confirmed" : "pending"}`, "Access path: pending"]}
      wizardSubtitle="Write this phrase down offline, then prove backup with selected words."
      wizardTitle="Recovery checkpoint"
    >
      <Notice className={styles.notice} suppressHydrationWarning tone="warning">
        {phrase.length === 12 ? phrase.join(" ") : "recovery_phrase_unavailable"}
      </Notice>

      <Field label="Enter word 3">
        <TextInput value={word3} onChange={(event) => setWord3(event.target.value)} />
      </Field>
      <Field label="Enter word 7">
        <TextInput value={word7} onChange={(event) => setWord7(event.target.value)} />
      </Field>
      <Field label="Enter word 11">
        <TextInput value={word11} onChange={(event) => setWord11(event.target.value)} />
      </Field>

      {confirmed ? (
        <Notice className={styles.notice} tone="success">
          Recovery backup status: confirmed.
        </Notice>
      ) : (
        <Notice className={styles.notice} tone="danger">
          recovery_unconfirmed: words do not match required positions.
        </Notice>
      )}

      <div className={styles.ctaRow}>
        <ButtonLink href="/onboarding/identity" variant="ghost">
          Back to identity
        </ButtonLink>
        <ButtonLink disabled={!confirmed} href="/onboarding/access" variant="primary">
          Continue to access
        </ButtonLink>
      </div>
    </Shell>
  );
}
