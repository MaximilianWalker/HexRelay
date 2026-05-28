import { ToggleSwitch } from "@/components/ui/field";

type SettingToggleProps = {
  checked: boolean;
  disabled?: boolean;
  label: string;
  onChange?: (next: boolean) => void;
};

export function SettingToggle({
  checked,
  disabled,
  label,
  onChange,
}: SettingToggleProps) {
  return (
    <ToggleSwitch
      checked={checked}
      disabled={disabled}
      label={label}
      onChange={onChange}
    />
  );
}
