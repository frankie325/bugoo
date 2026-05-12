import { Text, Description } from "@heroui/react";
import type { ReactNode } from "react";

interface SettingItemProps {
  title: string;
  description: string;
  children: ReactNode;
}

export function SettingItem({
  title,
  description,
  children,
}: SettingItemProps) {
  return (
    <div className="flex items-center justify-between py-1">
      <div>
        <Text type="h6">{title}</Text>
        <Description>{description}</Description>
      </div>
      <div>{children}</div>
    </div>
  );
}
