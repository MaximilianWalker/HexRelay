import { forwardRef } from "react";
import type { ButtonHTMLAttributes } from "react";

import { useToggleButton } from "../behavior";

type RootProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-pressed"> & {
  onPressedChange?: (pressed: boolean) => void;
  pressed?: boolean;
};

export const Root = forwardRef<HTMLButtonElement, RootProps>(function Root(
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
