"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useMemo, useState } from "react";

import styles from "./workspace-shell.module.css";

const SIDEBAR_MODE_KEY = "hexrelay.ui.sidebar-mode.v1";

type TabItem = {
  id: string;
  label: string;
};

export function WorkspaceShell({
  title,
  subtitle,
  tabs,
  activeTabId,
  children,
}: {
  title: string;
  subtitle: string;
  tabs: TabItem[];
  activeTabId: string;
  children: React.ReactNode;
}) {
  const pathname = usePathname();
  const [collapsed, setCollapsed] = useState(() => {
    if (typeof window === "undefined") {
      return false;
    }

    return window.localStorage.getItem(SIDEBAR_MODE_KEY) === "collapsed";
  });

  function toggleSidebar(): void {
    setCollapsed((value) => {
      const next = !value;
      window.localStorage.setItem(SIDEBAR_MODE_KEY, next ? "collapsed" : "expanded");
      return next;
    });
  }

  const nav = useMemo(
    () => [
      { href: "/home", label: "Home" },
      { href: "/servers", label: "Servers" },
      { href: "/contacts", label: "Contacts" },
      { href: "/settings", label: "Settings" },
    ],
    [],
  );

  const serverShortcuts = useMemo(
    () => [
      { label: "Atlas Core", unread: "2" },
      { label: "Relay Lab", unread: "0" },
      { label: "Dev Signals", unread: "5" },
      { label: "Ops Watch", unread: "0" },
    ],
    [],
  );

  return (
    <main className={`${styles.shell} ${collapsed ? styles.collapsed : ""}`}>
      <div className={styles.frame}>
        <aside className={styles.sidebar}>
          <p className={styles.brand}>HexRelay</p>
          <nav className={styles.topNav}>
            {nav.map((item) => {
              const active = pathname === item.href;
              return (
                <Link
                  className={`${styles.navLink} ${active ? styles.navLinkActive : ""}`}
                  href={item.href}
                  key={item.href}
                >
                  <span className={styles.navLabel}>{item.label}</span>
                </Link>
              );
            })}
          </nav>

          <p className={styles.sectionTitle}>Server sidebar mode</p>
          <div className={styles.serverList}>
            {serverShortcuts.map((server) => (
              <div className={styles.serverItem} key={server.label}>
                <span className={styles.serverLabel}>{server.label}</span>
                <span className={styles.serverUnread}>{server.unread}</span>
              </div>
            ))}
          </div>
        </aside>

        <section className={styles.content}>
          <header className={styles.header}>
            <div>
              <h1 className={styles.heading}>{title}</h1>
              <p className={styles.subtitle}>{subtitle}</p>
            </div>
            <button className={styles.burger} onClick={toggleSidebar} type="button">
              {collapsed ? "Expand sidebar" : "Collapse sidebar"}
            </button>
          </header>

          <div className={styles.tabs}>
            {tabs.map((tab) => (
              <div
                className={`${styles.tab} ${tab.id === activeTabId ? styles.tabActive : ""}`}
                key={tab.id}
              >
                {tab.label}
              </div>
            ))}
          </div>

          <section className={styles.body}>{children}</section>
        </section>
      </div>

      <nav className={styles.mobileTabs}>
        {nav.map((item) => {
          const active = pathname === item.href;
          return (
            <Link
              className={`${styles.mobileTab} ${active ? styles.mobileTabActive : ""}`}
              href={item.href}
              key={item.href}
            >
              {item.label}
            </Link>
          );
        })}
      </nav>
    </main>
  );
}
