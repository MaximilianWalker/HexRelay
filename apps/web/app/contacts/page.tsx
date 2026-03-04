import { WorkspaceShell } from "@/components/workspace-shell";

import styles from "../surfaces.module.css";

const CONTACTS = [
  { name: "Nora K", status: "online", unread: 1 },
  { name: "Alex R", status: "offline", unread: 0 },
  { name: "Mina S", status: "online", unread: 3 },
  { name: "Jules P", status: "away", unread: 0 },
];

export default function ContactsPage() {
  return (
    <WorkspaceShell
      activeTabId="contacts"
      subtitle="Contacts hub with friend-request and direct-message entrypoints"
      tabs={[
        { id: "contacts", label: "Contacts Hub" },
        { id: "friends", label: "Friends" },
        { id: "requests", label: "Requests" },
      ]}
      title="Contacts"
    >
      <section>
        <div className={styles.row}>
          <span className={styles.pill}>filter: online</span>
          <span className={styles.pill}>filter: unread</span>
          <span className={styles.pill}>filter: favorites</span>
        </div>
        <input className={styles.search} placeholder="Search contacts" readOnly value="" />
        <div className={styles.grid}>
          {CONTACTS.map((contact) => (
            <article className={styles.card} key={contact.name}>
              <p className={styles.title}>{contact.name}</p>
              <p className={styles.meta}>
                {contact.status} · unread {contact.unread}
              </p>
            </article>
          ))}
        </div>
        <p className={styles.state}>
          Mediated request state handling: `friend_request_pending`, `friend_request_inbound`, and `search_no_results`.
        </p>
      </section>
    </WorkspaceShell>
  );
}
