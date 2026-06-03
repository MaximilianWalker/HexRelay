import styles from "@/app/surfaces.module.css";
import { initials } from "@/lib/ui/initials";

type IconProps = {
  name: string;
};

export function Icon({ name }: IconProps) {
  return (
    <div className={styles.serverImage} aria-label={`${name} icon`} role="img">
      <span>{initials(name)}</span>
    </div>
  );
}
