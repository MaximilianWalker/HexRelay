import { ToggleGroup, type ToggleGroupOption } from "./toggle-group";

export type SegmentedControlOption<T extends string> = ToggleGroupOption<T>;

export function SegmentedControl<T extends string>({
  label,
  onChange,
  options,
  value,
}: {
  label: string;
  onChange: (value: T) => void;
  options: SegmentedControlOption<T>[];
  value: T;
}) {
  return <ToggleGroup label={label} onChange={onChange} options={options} value={value} />;
}
