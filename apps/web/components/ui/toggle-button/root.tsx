import type { ComponentProps } from "react";

import { Button } from "../button";
import { useToggleButton } from "../behavior";

type ToggleButtonProps = Omit<ComponentProps<typeof Button>, "aria-pressed" | "pressed"> & {
  onPressedChange?: (pressed: boolean) => void;
  pressed: boolean;
};

export function Root({
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
