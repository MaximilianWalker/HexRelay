import { ToggleSwitch } from "@/components/ui/toggles/toggle-switch";

type ToggleProps = {
  checked: boolean;
  disabled?: boolean;
  label: string;
  onChange?: (next: boolean) => void;
};

export function Toggle({
  checked,
  disabled,
  label,
  onChange,
}: ToggleProps) {
  return (
    <ToggleSwitch
      checked={checked}
      disabled={disabled}
      label={label}
      onChange={onChange}
    />
  );
}
