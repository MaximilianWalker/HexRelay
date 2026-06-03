import { useCallback } from "react";
import type { ButtonHTMLAttributes, MouseEvent } from "react";

type ToggleButtonBehaviorOptions = {
  disabled?: boolean;
  onClick?: ButtonHTMLAttributes<HTMLButtonElement>["onClick"];
  onPressedChange?: (pressed: boolean) => void;
  pressed: boolean;
};

export function useToggleButton({
  disabled,
  onClick,
  onPressedChange,
  pressed,
}: ToggleButtonBehaviorOptions): Pick<ButtonHTMLAttributes<HTMLButtonElement>, "aria-pressed" | "disabled" | "onClick"> {
  const handleClick = useCallback(
    (event: MouseEvent<HTMLButtonElement>) => {
      onClick?.(event);

      if (event.defaultPrevented || disabled) {
        return;
      }

      onPressedChange?.(!pressed);
    },
    [disabled, onClick, onPressedChange, pressed],
  );

  return {
    "aria-pressed": pressed,
    disabled,
    onClick: handleClick,
  };
}
