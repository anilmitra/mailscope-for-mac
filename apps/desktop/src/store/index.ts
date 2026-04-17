import { create } from "zustand";
import type {
  ConnectedAccount,
  SeedGroup,
  SendingProfile,
  TestRun,
  DashboardStats,
} from "../types";

interface AppState {
  accounts: ConnectedAccount[];
  seedGroups: SeedGroup[];
  sendingProfiles: SendingProfile[];
  activeRun: TestRun | null;
  dashboardStats: DashboardStats | null;

  setAccounts: (accounts: ConnectedAccount[]) => void;
  upsertAccount: (account: ConnectedAccount) => void;
  removeAccount: (id: number) => void;

  setSeedGroups: (groups: SeedGroup[]) => void;
  upsertSeedGroup: (group: SeedGroup) => void;
  removeSeedGroup: (id: number) => void;

  setSendingProfiles: (profiles: SendingProfile[]) => void;
  upsertSendingProfile: (profile: SendingProfile) => void;
  removeSendingProfile: (id: number) => void;

  setActiveRun: (run: TestRun | null) => void;
  setDashboardStats: (stats: DashboardStats) => void;
}

export const useAppStore = create<AppState>((set) => ({
  accounts: [],
  seedGroups: [],
  sendingProfiles: [],
  activeRun: null,
  dashboardStats: null,

  setAccounts: (accounts) => set({ accounts }),
  upsertAccount: (account) =>
    set((s) => ({
      accounts: s.accounts.some((a) => a.id === account.id)
        ? s.accounts.map((a) => (a.id === account.id ? account : a))
        : [...s.accounts, account],
    })),
  removeAccount: (id) =>
    set((s) => ({ accounts: s.accounts.filter((a) => a.id !== id) })),

  setSeedGroups: (seedGroups) => set({ seedGroups }),
  upsertSeedGroup: (group) =>
    set((s) => ({
      seedGroups: s.seedGroups.some((g) => g.id === group.id)
        ? s.seedGroups.map((g) => (g.id === group.id ? group : g))
        : [...s.seedGroups, group],
    })),
  removeSeedGroup: (id) =>
    set((s) => ({ seedGroups: s.seedGroups.filter((g) => g.id !== id) })),

  setSendingProfiles: (sendingProfiles) => set({ sendingProfiles }),
  upsertSendingProfile: (profile) =>
    set((s) => ({
      sendingProfiles: s.sendingProfiles.some((p) => p.id === profile.id)
        ? s.sendingProfiles.map((p) => (p.id === profile.id ? profile : p))
        : [...s.sendingProfiles, profile],
    })),
  removeSendingProfile: (id) =>
    set((s) => ({
      sendingProfiles: s.sendingProfiles.filter((p) => p.id !== id),
    })),

  setActiveRun: (activeRun) => set({ activeRun }),
  setDashboardStats: (dashboardStats) => set({ dashboardStats }),
}));
