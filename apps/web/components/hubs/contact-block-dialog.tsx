import { Button } from "@/components/ui/button";
import { Dialog, DialogActions } from "@/components/ui/dialog";

type ContactBlockDialogProps = {
  busy: boolean;
  onClose: () => void;
  onConfirm: () => void;
  targetLabel: string;
};

export function ContactBlockDialog({ busy, onClose, onConfirm, targetLabel }: ContactBlockDialogProps) {
  return (
    <Dialog
      description="This blocks the user, removes the contact relationship, and keeps existing DM history."
      onClose={onClose}
      title={`Block + Remove ${targetLabel}?`}
    >
      <DialogActions>
        <Button disabled={busy} onClick={onClose}>
          Cancel
        </Button>
        <Button disabled={busy} onClick={onConfirm} variant="danger">
          Block + Remove
        </Button>
      </DialogActions>
    </Dialog>
  );
}
