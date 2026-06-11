import type { ComponentProps } from "react";

import { Button } from "../buttons/button";
import { useToggleButton } from "../buttons/behavior";

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
