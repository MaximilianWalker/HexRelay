import type { ButtonHTMLAttributes, ReactNode } from "react";

import { Button as UiButton } from "@/components/ui/buttons/button";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  children: ReactNode;
  variant?: "primary" | "secondary";
};

export function Button({
  children,
  variant = "secondary",
  ...props
}: ButtonProps) {
  return (
    <UiButton size="sm" variant={variant} {...props}>
      {children}
    </UiButton>
  );
}
