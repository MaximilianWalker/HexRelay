import type { ButtonHTMLAttributes, ReactNode } from "react";

import { Button } from "@/components/ui/button";

type SettingButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  children: ReactNode;
  variant?: "primary" | "secondary";
};

export function SettingButton({
  children,
  variant = "secondary",
  ...props
}: SettingButtonProps) {
  return (
    <Button size="sm" variant={variant} {...props}>
      {children}
    </Button>
  );
}
