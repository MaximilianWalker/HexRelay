import { IconSearch, IconUserPlus } from "@tabler/icons-react";

import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { DialogActions } from "@/components/ui/dialog-actions";
import { Field } from "@/components/ui/field";
import { Alert } from "@/components/ui/alert";
import { TextInput } from "@/components/ui/text-input";

import { ContactDiscoveryResults, type ContactDiscoveryUser } from "./contact-discovery-results";
import styles from "./styles.module.css";

type ContactAddDialogProps = {
  actionMessage: string | null;
  discoveryBusy: boolean;
  onClose: () => void;
  onQueryChange: (value: string) => void;
  onSearchUsers: () => void;
  onSendFriendRequest: (identityId: string) => void;
  query: string;
  sendBusyIdentityId: string | null;
  shortIdentity: (identityId: string) => string;
  users: ContactDiscoveryUser[];
};

export function ContactAddDialog({
  actionMessage,
  discoveryBusy,
  onClose,
  onQueryChange,
  onSearchUsers,
  onSendFriendRequest,
  query,
  sendBusyIdentityId,
  shortIdentity,
  users,
}: ContactAddDialogProps) {
  return (
    <Dialog
      description="Search by display name or identity id, then send a friend request from the result list."
      onClose={onClose}
      title="Add contact"
    >
      <div className={styles.dialogStack}>
        <Field label="Name or identity id">
          <div className={styles.searchWrap}>
            <IconSearch className={styles.searchIcon} aria-hidden="true" />
            <TextInput
              aria-label="User search or identity id"
              autoComplete="off"
              data-autofocus
              onChange={(event) => onQueryChange(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  event.preventDefault();
                  onSearchUsers();
                }
              }}
              placeholder="alice.primary or usr_..."
              value={query}
            />
          </div>
        </Field>
        {actionMessage ? <Alert>{actionMessage}</Alert> : null}
        <DialogActions>
          <Button
            disabled={discoveryBusy}
            icon={<IconSearch aria-hidden="true" />}
            onClick={onSearchUsers}
          >
            {discoveryBusy ? "Searching..." : "Search"}
          </Button>
          <Button
            disabled={sendBusyIdentityId === query.trim()}
            icon={<IconUserPlus aria-hidden="true" />}
            onClick={() => onSendFriendRequest(query)}
            variant="primary"
          >
            Send request
          </Button>
        </DialogActions>
        <ContactDiscoveryResults
          onSendFriendRequest={onSendFriendRequest}
          sendBusyIdentityId={sendBusyIdentityId}
          shortIdentity={shortIdentity}
          users={users}
        />
      </div>
    </Dialog>
  );
}
