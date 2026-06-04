import { ToggleGroup, type ToggleGroupOption, type ToggleGroupSize } from "./toggle-group";

export type ButtonGroupOption<T extends string> = ToggleGroupOption<T>;
export type ButtonGroupSize = ToggleGroupSize;

export function ButtonGroup<T extends string>({
  label,
  onChange,
  options,
  size,
  value,
}: {
  label: string;
  onChange: (value: T) => void;
  options: ButtonGroupOption<T>[];
  size?: ButtonGroupSize;
  value: T;
}) {
  return <ToggleGroup label={label} onChange={onChange} options={options} size={size} value={value} />;
}
