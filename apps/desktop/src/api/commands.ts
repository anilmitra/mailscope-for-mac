import { invoke } from "@tauri-apps/api/core";
import type {
  ConnectedAccount,
  SeedGroup,
  SeedGroupMember,
  SendingProfile,
  TestRun,
  TestResult,
  DiagnosticIssue,
  RunToken,
  DashboardStats,
  Provider,
  AuthType,
} from "../types";

// ── Accounts ──────────────────────────────────────────────────────────────────

export const listAccounts = () =>
  invoke<ConnectedAccount[]>("list_accounts");

export const getAccount = (id: number) =>
  invoke<ConnectedAccount>("get_account", { id });

export const addImapAccount = (params: {
  displayName: string;
  seedEmail: string;
  host: string;
  port: number;
  authType: AuthType;
  password: string;
}) => invoke<ConnectedAccount>("add_imap_account", { params });

export const startGmailOAuth = () =>
  invoke<string>("start_gmail_oauth");

export const completeGmailOAuth = (code: string, state: string) =>
  invoke<ConnectedAccount>("complete_gmail_oauth", { code, state });

export const startMicrosoftOAuth = () =>
  invoke<string>("start_microsoft_oauth");

export const completeMicrosoftOAuth = (code: string, state: string) =>
  invoke<ConnectedAccount>("complete_microsoft_oauth", { code, state });

export const removeAccount = (id: number) =>
  invoke<void>("remove_account", { id });

export const checkAccountHealth = (id: number) =>
  invoke<ConnectedAccount>("check_account_health", { id });

// ── Seed Groups ───────────────────────────────────────────────────────────────

export const listSeedGroups = () =>
  invoke<SeedGroup[]>("list_seed_groups");

export const createSeedGroup = (name: string, description?: string) =>
  invoke<SeedGroup>("create_seed_group", { name, description });

export const deleteSeedGroup = (id: number) =>
  invoke<void>("delete_seed_group", { id });

export const addAccountToGroup = (seedGroupId: number, accountId: number) =>
  invoke<SeedGroupMember>("add_account_to_group", { seedGroupId, accountId });

export const removeAccountFromGroup = (seedGroupId: number, accountId: number) =>
  invoke<void>("remove_account_from_group", { seedGroupId, accountId });

export const getSeedGroupMembers = (seedGroupId: number) =>
  invoke<SeedGroupMember[]>("get_seed_group_members", { seedGroupId });

// ── Sending Profiles ──────────────────────────────────────────────────────────

export const listSendingProfiles = () =>
  invoke<SendingProfile[]>("list_sending_profiles");

export const createSendingProfile = (params: {
  name: string;
  fromName: string;
  fromEmail: string;
  replyTo?: string;
  notes?: string;
}) => invoke<SendingProfile>("create_sending_profile", { params });

export const deleteSendingProfile = (id: number) =>
  invoke<void>("delete_sending_profile", { id });

// ── Test Runs ─────────────────────────────────────────────────────────────────

export const listRuns = (limit?: number) =>
  invoke<TestRun[]>("list_runs", { limit });

export const getRun = (id: number) =>
  invoke<TestRun>("get_run", { id });

export const generateRunToken = (sendingProfileId: number, subjectTemplate: string) =>
  invoke<RunToken>("generate_run_token", { sendingProfileId, subjectTemplate });

export const createRun = (params: {
  sendingProfileId: number;
  seedGroupId: number;
  subjectTemplate: string;
  bodyText: string;
  bodyHtml?: string;
  subjectToken: string;
  bodyToken: string;
}) => invoke<TestRun>("create_run", { params });

export const startRun = (id: number) =>
  invoke<TestRun>("start_run", { id });

export const cancelRun = (id: number) =>
  invoke<void>("cancel_run", { id });

// ── Results ───────────────────────────────────────────────────────────────────

export const getRunResults = (runId: number) =>
  invoke<TestResult[]>("get_run_results", { runId });

export const getRunDiagnostics = (runId: number) =>
  invoke<DiagnosticIssue[]>("get_run_diagnostics", { runId });

export const exportRunCsv = (runId: number) =>
  invoke<string>("export_run_csv", { runId });

export const exportRunJson = (runId: number) =>
  invoke<string>("export_run_json", { runId });

// ── Dashboard ─────────────────────────────────────────────────────────────────

export const getDashboardStats = () =>
  invoke<DashboardStats>("get_dashboard_stats");

// ── DNS / Preflight ───────────────────────────────────────────────────────────

export const checkDomain = (domain: string) =>
  invoke<{
    spf: string | null;
    dkim: string | null;
    dmarc: string | null;
    hasListUnsubscribe: boolean;
  }>("check_domain", { domain });

export const lookupProviderByEmail = (email: string) =>
  invoke<Provider>("lookup_provider_by_email", { email });
