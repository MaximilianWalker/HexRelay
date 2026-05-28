import { IconInfoCircle, IconX } from "@tabler/icons-react";

import { Avatar } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Panel } from "@/components/ui/panel";
import type { PersonaRecord } from "@/lib/personas";

import styles from "./hubs.module.css";

export type ContactRequest = {
  created_at?: string;
  request_id: string;
  requester_identity_id: string;
  status: string;
  target_identity_id: string;
};

type ContactRequestSectionProps = {
  busyRequestId: string | null;
  formatDateTime: (value?: string) => string;
  identityId: string;
  identityLabel: (identityId: string, activeIdentityId: string, personas: PersonaRecord[]) => string;
  kind: "inbound" | "outbound";
  onTransition: (requestId: string, action: "accept" | "decline" | "cancel") => Promise<void>;
  personas: PersonaRecord[];
  requests: ContactRequest[];
};

export function ContactRequestSection({
  busyRequestId,
  formatDateTime,
  identityId,
  identityLabel,
  kind,
  onTransition,
  personas,
  requests,
}: ContactRequestSectionProps) {
  return (
    <Panel aria-label={kind === "inbound" ? "Friend requests" : "Sent requests"} className={styles.requestSection}>
      <p className={styles.requestTitle}>{kind === "inbound" ? "Friend requests" : "Sent requests"}</p>
      <div className={styles.requestGrid}>
        {requests.map((request) => {
          const peerId = kind === "inbound" ? request.requester_identity_id : request.target_identity_id;
          const peerName = identityLabel(peerId, identityId, personas);

          return (
            <article className={styles.requestCard} key={request.request_id}>
              <div className={styles.requestHeader}>
                <Avatar kind="user" text={initials(peerName)} />
                <div className={styles.requestPeer}>
                  <p>{peerName}</p>
                  <span>{kind === "inbound" ? "Wants to add you" : "Waiting for them to accept"}</span>
                </div>
              </div>
              <div className={styles.requestBadges}>
                <Badge tone={kind === "inbound" ? "accent" : "muted"}>
                  {kind === "inbound" ? "Needs your approval" : "Pending"}
                </Badge>
                {request.created_at ? <Badge tone="muted">Sent {formatDateTime(request.created_at)}</Badge> : null}
              </div>
              <div className={styles.requestActions}>
                {kind === "inbound" ? (
                  <>
                    <Button
                      disabled={busyRequestId === request.request_id}
                      icon={<IconInfoCircle className={styles.icon} aria-hidden="true" />}
                      onClick={() => void onTransition(request.request_id, "accept")}
                      size="sm"
                    >
                      Accept
                    </Button>
                    <Button
                      disabled={busyRequestId === request.request_id}
                      icon={<IconX className={styles.icon} aria-hidden="true" />}
                      onClick={() => void onTransition(request.request_id, "decline")}
                      size="sm"
                    >
                      Decline
                    </Button>
                  </>
                ) : (
                  <Button
                    disabled={busyRequestId === request.request_id}
                    icon={<IconX className={styles.icon} aria-hidden="true" />}
                    onClick={() => void onTransition(request.request_id, "cancel")}
                    size="sm"
                  >
                    Cancel
                  </Button>
                )}
              </div>
            </article>
          );
        })}
      </div>
    </Panel>
  );
}

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return "?";
  }

  return parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}
