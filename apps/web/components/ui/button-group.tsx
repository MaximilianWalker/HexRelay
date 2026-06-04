import { ToggleGroup, type ToggleGroupOption } from "./toggle-group";

export type ButtonGroupOption<T extends string> = ToggleGroupOption<T>;

export function ButtonGroup<T extends string>({
  label,
  onChange,
  options,
  value,
}: {
  label: string;
  onChange: (value: T) => void;
  options: ButtonGroupOption<T>[];
  value: T;
}) {
  return <ToggleGroup label={label} onChange={onChange} options={options} value={value} />;
}
