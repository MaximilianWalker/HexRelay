import type { ComponentProps } from "react";

import { useToggleButton } from "./behavior";
import { Button } from "./button";

type ToggleButtonProps = Omit<ComponentProps<typeof Button>, "aria-pressed" | "pressed"> & {
  onPressedChange?: (pressed: boolean) => void;
  pressed: boolean;
};

export function ToggleButton({
  disabled,
  onClick,
  onPressedChange,
  pressed,
  ...props
}: ToggleButtonProps) {
  const buttonProps = useToggleButton({
    disabled,
    onClick,
    onPressedChange,
    pressed,
  });

  return <Button {...props} {...buttonProps} pressed={pressed} />;
}
