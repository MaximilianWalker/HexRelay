import styles from "@/app/surfaces.module.css";

type ServerIconProps = {
  name: string;
};

function initials(value: string): string {
  const parts = value.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return "?";
  }

  return parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

export function ServerIcon({ name }: ServerIconProps) {
  return (
    <div className={styles.serverImage} aria-label={`${name} icon`} role="img">
      <span>{initials(name)}</span>
    </div>
  );
}
