import { Button } from "@/components/ui/button";
import { CheckboxField } from "@/components/ui/checkbox-field";
import { Dialog } from "@/components/ui/dialog";
import { DialogActions } from "@/components/ui/dialog-actions";
import { Field } from "@/components/ui/field";
import { Notice } from "@/components/ui/notice";
import { TextInput } from "@/components/ui/text-input";

import styles from "./styles.module.css";

type ServerJoinDialogProps = {
  actionMessage: string | null;
  busy: boolean;
  endpoint: string;
  inviteLink: string;
  inviteToken: string;
  onClose: () => void;
  onEndpointChange: (value: string) => void;
  onInviteLinkChange: (value: string) => void;
  onInviteTokenChange: (value: string) => void;
  onServerIdChange: (value: string) => void;
  onShowAdvancedChange: (value: boolean) => void;
  onSubmit: () => void;
  serverId: string;
  showAdvanced: boolean;
};

export function ServerJoinDialog({
  actionMessage,
  busy,
  endpoint,
  inviteLink,
  inviteToken,
  onClose,
  onEndpointChange,
  onInviteLinkChange,
  onInviteTokenChange,
  onServerIdChange,
  onShowAdvancedChange,
  onSubmit,
  serverId,
  showAdvanced,
}: ServerJoinDialogProps) {
  return (
    <Dialog
      description="Paste an invite link, or use advanced fields when you have a server endpoint and invite token."
      onClose={onClose}
      title="Join server"
    >
      <form
        className={styles.dialogForm}
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
      >
        <Field label="Invite link">
          <TextInput
            autoComplete="off"
            data-autofocus
            onChange={(event) => onInviteLinkChange(event.target.value)}
            placeholder="hexrelay://invite/..."
            value={inviteLink}
          />
        </Field>
        <CheckboxField checked={showAdvanced} onChange={(event) => onShowAdvancedChange(event.target.checked)}>
          Show advanced fields
        </CheckboxField>
        {showAdvanced ? (
          <>
            <Field label="Endpoint">
              <TextInput
                onChange={(event) => onEndpointChange(event.target.value)}
                placeholder="https://server.example"
                value={endpoint}
              />
            </Field>
            <Field label="Server id">
              <TextInput
                onChange={(event) => onServerIdChange(event.target.value)}
                placeholder="srv_..."
                value={serverId}
              />
            </Field>
            <Field label="Invite token">
              <TextInput
                onChange={(event) => onInviteTokenChange(event.target.value)}
                placeholder="Invite token"
                value={inviteToken}
              />
            </Field>
          </>
        ) : null}
        {actionMessage ? <Notice>{actionMessage}</Notice> : null}
        <DialogActions>
          <Button disabled={busy} onClick={onClose}>
            Cancel
          </Button>
          <Button disabled={busy} type="submit" variant="primary">
            Join server
          </Button>
        </DialogActions>
      </form>
    </Dialog>
  );
}
