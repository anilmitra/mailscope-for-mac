export type Provider = "gmail" | "microsoft" | "imap";
export type AuthType = "oauth" | "password" | "app_password";
export type AccountStatus = "active" | "error" | "expired" | "unknown";
export type SendingMode = "manual" | "smtp" | "api";
export type RunStatus = "draft" | "running" | "complete" | "failed";

export type Placement =
  | "inbox"
  | "spam"
  | "junk"
  | "promotions"
  | "social"
  | "updates"
  | "missing"
  | "other";

export type DiagnosticSeverity = "info" | "warning" | "error" | "critical";

export type CheckType =
  | "spf"
  | "dkim"
  | "dmarc"
  | "dns"
  | "blocklist"
  | "content"
  | "list_unsubscribe"
  | "latency";

export interface ConnectedAccount {
  id: number;
  provider: Provider;
  displayName: string;
  seedEmail: string;
  authType: AuthType;
  keychainRef: string;
  status: AccountStatus;
  lastSeenAt: string | null;
  createdAt: string;
}

export interface SeedGroup {
  id: number;
  name: string;
  description: string | null;
  createdAt: string;
  memberCount?: number;
}

export interface SeedGroupMember {
  id: number;
  seedGroupId: number;
  connectedAccountId: number;
  account?: ConnectedAccount;
}

export interface SendingProfile {
  id: number;
  name: string;
  fromName: string;
  fromEmail: string;
  replyTo: string | null;
  sendingMode: SendingMode;
  smtpConfigRef: string | null;
  notes: string | null;
  createdAt: string;
}

export interface TestRun {
  id: number;
  runUuid: string;
  sendingProfileId: number;
  seedGroupId: number;
  status: RunStatus;
  subjectTemplate: string;
  bodyText: string;
  bodyHtml: string | null;
  subjectToken: string;
  bodyToken: string;
  startedAt: string | null;
  completedAt: string | null;
  sendingProfile?: SendingProfile;
  seedGroup?: SeedGroup;
  results?: TestResult[];
  diagnostics?: DiagnosticIssue[];
  inboxRate?: number;
  spamRate?: number;
  missingRate?: number;
}

export interface TestResult {
  id: number;
  testRunId: number;
  connectedAccountId: number;
  provider: Provider;
  placement: Placement;
  rawFolder: string | null;
  messageIdRemote: string | null;
  matchedAt: string | null;
  deliveryLatencyMs: number | null;
  spfResult: string | null;
  dkimResult: string | null;
  dmarcResult: string | null;
  authSummary: string | null;
  headersJson: Record<string, string> | null;
  notesJson: string[] | null;
  account?: ConnectedAccount;
}

export interface DiagnosticIssue {
  id: number;
  testRunId: number;
  checkType: CheckType;
  severity: DiagnosticSeverity;
  title: string;
  details: string;
  recommendation: string;
}

export interface RunToken {
  subjectToken: string;
  bodyToken: string;
  suggestedSubject: string;
  suggestedBody: string;
}

export interface PlacementSummary {
  provider: Provider;
  inbox: number;
  spam: number;
  junk: number;
  promotions: number;
  social: number;
  missing: number;
  other: number;
  total: number;
  inboxRate: number;
}

export interface DashboardStats {
  lastRun: TestRun | null;
  totalRuns: number;
  avgInboxRate: number;
  avgSpamRate: number;
  avgMissingRate: number;
  recentRuns: TestRun[];
  providerSummary: PlacementSummary[];
}
