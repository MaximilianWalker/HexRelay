import styles from "@/app/surfaces.module.css";
import { initials } from "@/lib/ui/initials";

type ServerIconProps = {
  name: string;
};

export function ServerIcon({ name }: ServerIconProps) {
  return (
    <div className={styles.serverImage} aria-label={`${name} icon`} role="img">
      <span>{initials(name)}</span>
    </div>
  );
}
