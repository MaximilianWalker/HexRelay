import { Button } from "@/components/ui/button";
import { CheckboxField } from "@/components/ui/checkbox-field";
import { Dialog } from "@/components/ui/dialog";
import { DialogActions } from "@/components/ui/dialog-actions";

import styles from "./hubs.module.css";

type ServerLeaveDialogProps = {
  busy: boolean;
  deleteLocalData: boolean;
  onClose: () => void;
  onConfirm: () => void;
  onDeleteLocalDataChange: (value: boolean) => void;
  targetLabel: string;
};

export function ServerLeaveDialog({
  busy,
  deleteLocalData,
  onClose,
  onConfirm,
  onDeleteLocalDataChange,
  targetLabel,
}: ServerLeaveDialogProps) {
  return (
    <Dialog
      description="Leaving removes the server from this hub and closes related workspace tabs."
      onClose={onClose}
      title={`Leave ${targetLabel}?`}
    >
      <div className={styles.dialogStack}>
        <CheckboxField checked={deleteLocalData} onChange={(event) => onDeleteLocalDataChange(event.target.checked)}>
          Delete local data for this server
        </CheckboxField>
        <DialogActions>
          <Button disabled={busy} onClick={onClose}>
            Cancel
          </Button>
          <Button disabled={busy} onClick={onConfirm} variant="danger">
            Leave server
          </Button>
        </DialogActions>
      </div>
    </Dialog>
  );
}
