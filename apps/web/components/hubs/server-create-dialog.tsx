import { Button } from "@/components/ui/button";
import { CheckboxField } from "@/components/ui/checkbox-field";
import { Dialog } from "@/components/ui/dialog";
import { DialogActions } from "@/components/ui/dialog-actions";
import { Field } from "@/components/ui/field";
import { Notice } from "@/components/ui/notice";
import { TextInput } from "@/components/ui/text-input";

import styles from "./hubs.module.css";

type ServerCreateDialogProps = {
  actionMessage: string | null;
  bootstrapCredential: string;
  busy: boolean;
  description: string;
  manualBootstrap: boolean;
  name: string;
  onBootstrapCredentialChange: (value: string) => void;
  onClose: () => void;
  onDescriptionChange: (value: string) => void;
  onManualBootstrapChange: (value: boolean) => void;
  onNameChange: (value: string) => void;
  onSubmit: () => void;
};

export function ServerCreateDialog({
  actionMessage,
  bootstrapCredential,
  busy,
  description,
  manualBootstrap,
  name,
  onBootstrapCredentialChange,
  onClose,
  onDescriptionChange,
  onManualBootstrapChange,
  onNameChange,
  onSubmit,
}: ServerCreateDialogProps) {
  return (
    <Dialog
      description="Create a local test server and choose whether to provide the bootstrap credential yourself."
      onClose={onClose}
      title="Create server"
    >
      <form
        className={styles.dialogForm}
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
      >
        <Field label="Server name">
          <TextInput
            autoComplete="off"
            data-autofocus
            onChange={(event) => onNameChange(event.target.value)}
            placeholder="Atlas Team"
            value={name}
          />
        </Field>
        <Field label="Description">
          <TextInput
            autoComplete="off"
            onChange={(event) => onDescriptionChange(event.target.value)}
            placeholder="Shared workspace for a team or community"
            value={description}
          />
        </Field>
        <CheckboxField checked={manualBootstrap} onChange={(event) => onManualBootstrapChange(event.target.checked)}>
          Supply bootstrap credential manually
        </CheckboxField>
        {manualBootstrap ? (
          <Field label="Bootstrap credential">
            <TextInput
              autoComplete="off"
              onChange={(event) => onBootstrapCredentialChange(event.target.value)}
              placeholder="Credential"
              value={bootstrapCredential}
            />
          </Field>
        ) : null}
        {actionMessage ? <Notice>{actionMessage}</Notice> : null}
        <DialogActions>
          <Button disabled={busy} onClick={onClose}>
            Cancel
          </Button>
          <Button disabled={busy} type="submit" variant="primary">
            Create server
          </Button>
        </DialogActions>
      </form>
    </Dialog>
  );
}
