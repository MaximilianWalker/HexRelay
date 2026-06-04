import { IconSend } from "@tabler/icons-react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { TextArea } from "@/components/ui/text-area";

import styles from "./styles.module.css";

export function Composer({
  disabled,
  hints,
  onCancelReply,
  onChange,
  onSend,
  placeholder,
  replyLabel,
  replyText,
  sendLabel,
  value,
}: {
  disabled?: boolean;
  hints: ReactNode;
  onCancelReply?: () => void;
  onChange: (value: string) => void;
  onSend: () => void;
  placeholder: string;
  replyLabel?: string;
  replyText?: string;
  sendLabel: string;
  value: string;
}) {
  return (
    <section className={styles.composerPanel} aria-label="Message composer">
      {replyLabel && replyText && onCancelReply ? (
        <div className={styles.replyDraft}>
          <div>
            <p className={styles.replyLabel}>{replyLabel}</p>
            <p className={styles.replyText}>{replyText}</p>
          </div>
          <Button onClick={onCancelReply}>Cancel reply</Button>
        </div>
      ) : null}

      <TextArea
        className={styles.composerInput}
        disabled={disabled}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        rows={3}
        value={value}
      />
      <div className={styles.composerBar}>
        <div className={styles.composerHints}>{hints}</div>
        <Button disabled={disabled} icon={<IconSend aria-hidden="true" />} onClick={onSend} variant="primary">
          {sendLabel}
        </Button>
      </div>
    </section>
  );
}
