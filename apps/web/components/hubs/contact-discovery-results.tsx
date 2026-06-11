import { IconUserPlus } from "@tabler/icons-react";

import { Avatar } from "@/components/ui/display/avatar";
import { Badge } from "@/components/ui/display/badge";
import { Button } from "@/components/ui/buttons/button";
import { Panel } from "@/components/ui/surfaces/panel";
import { initials } from "@/lib/ui/initials";

import styles from "./styles.module.css";

export type ContactDiscoveryUser = {
  can_send_friend_request: boolean;
  display_name: string;
  has_pending_inbound_request: boolean;
  has_pending_outbound_request: boolean;
  identity_id: string;
  relationship_state: string;
  shared_server_count: number;
};

type ContactDiscoveryResultsProps = {
  onSendFriendRequest: (identityId: string) => void;
  sendBusyIdentityId: string | null;
  shortIdentity: (identityId: string) => string;
  users: ContactDiscoveryUser[];
};

export function ContactDiscoveryResults({
  onSendFriendRequest,
  sendBusyIdentityId,
  shortIdentity,
  users,
}: ContactDiscoveryResultsProps) {
  if (users.length === 0) {
    return null;
  }

  return (
    <div className={styles.discoveryGrid}>
      {users.map((user) => (
        <Panel className={styles.discoveryCard} key={user.identity_id}>
          <div className={styles.discoveryHeader}>
            <Avatar kind="user" text={initials(user.display_name)} />
            <div className={styles.discoveryCopy}>
              <p>{user.display_name}</p>
              <span>{shortIdentity(user.identity_id)}</span>
            </div>
          </div>
          <div className={styles.discoveryBadges}>
            {user.shared_server_count > 0 ? <Badge tone="muted">{user.shared_server_count} shared servers</Badge> : null}
            {user.has_pending_outbound_request ? <Badge tone="muted">Request pending</Badge> : null}
            {user.has_pending_inbound_request ? <Badge tone="accent">Needs approval</Badge> : null}
          </div>
          <Button
            disabled={!user.can_send_friend_request || sendBusyIdentityId === user.identity_id}
            icon={<IconUserPlus aria-hidden="true" />}
            onClick={() => onSendFriendRequest(user.identity_id)}
            size="sm"
          >
            {user.can_send_friend_request ? "Send request" : "Unavailable"}
          </Button>
        </Panel>
      ))}
    </div>
  );
}
