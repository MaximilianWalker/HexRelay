export const CARGO_AUDIT_VERSION = process.env.CARGO_AUDIT_VERSION || "0.22.0";

export const cargoAuditAdvisories = [
  {
    id: "RUSTSEC-2023-0071",
    expires: "2026-06-30",
    rationale: "Temporary transitive advisory exception tracked by the security gate.",
  },
  {
    id: "RUSTSEC-2026-0049",
    expires: "2026-09-30",
    rationale: "Temporary transitive advisory exception tracked by the security gate.",
  },
  {
    id: "RUSTSEC-2026-0097",
    expires: "2026-08-31",
    rationale: "Temporary transitive advisory exception tracked by the security gate.",
  },
];

export function cargoAuditIgnoreArgs() {
  return cargoAuditAdvisories.flatMap((advisory) => ["--ignore", advisory.id]);
}
