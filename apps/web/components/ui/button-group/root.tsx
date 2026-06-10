import { ToggleGroup, type ToggleGroupOption, type ToggleGroupSize } from "../toggle-group";

export type Option<T extends string> = ToggleGroupOption<T>;
export type Size = ToggleGroupSize;

export function Root<T extends string>({
  label,
  onChange,
  options,
  size,
  value,
}: {
  label: string;
  onChange: (value: T) => void;
  options: Option<T>[];
  size?: Size;
  value: T;
}) {
  return <ToggleGroup label={label} onChange={onChange} options={options} size={size} value={value} />;
}
