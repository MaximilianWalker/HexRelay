import type { ComponentProps, ReactNode } from "react";

import { Button } from "./button";

export function IconButton({
  children,
  label,
  title,
  ...props
}: Omit<ComponentProps<typeof Button>, "aria-label" | "children" | "icon" | "size"> & {
  children: ReactNode;
  label: string;
}) {
  return (
    <Button aria-label={label} size="icon" title={title ?? label} {...props}>
      {children}
    </Button>
  );
}
