import { forwardRef } from "react";
import type { ButtonHTMLAttributes } from "react";

import { useToggleButton } from "./behavior";

type PressableButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-pressed"> & {
  onPressedChange?: (pressed: boolean) => void;
  pressed: boolean;
};

export const PressableButton = forwardRef<HTMLButtonElement, PressableButtonProps>(function PressableButton(
  { disabled, onClick, onPressedChange, pressed, type = "button", ...props },
  ref,
) {
  const buttonProps = useToggleButton({
    disabled,
    onClick,
    onPressedChange,
    pressed,
  });

  return <button ref={ref} type={type} {...props} {...buttonProps} />;
});
