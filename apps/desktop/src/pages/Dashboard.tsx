import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import {
  CheckCircle2,
  AlertTriangle,
  XCircle,
  HelpCircle,
  ArrowRight,
  Play,
} from "lucide-react";
import { getDashboardStats } from "../api/commands";
import type { PlacementSummary, TestRun } from "../types";

function StatCard({
  label,
  value,
  color,
  icon: Icon,
}: {
  label: string;
  value: string;
  color: string;
  icon: React.ElementType;
}) {
  return (
    <div className="card flex items-start gap-3">
      <div className={`p-2 rounded-lg ${color} bg-opacity-20`}>
        <Icon className={`w-4 h-4 ${color.replace("bg-", "text-")}`} />
      </div>
      <div>
        <div className="text-xl font-semibold text-white">{value}</div>
        <div className="text-xs text-white/50 mt-0.5">{label}</div>
      </div>
    </div>
  );
}

function ProviderCard({ summary }: { summary: PlacementSummary }) {
  const providerLabel: Record<string, string> = {
    gmail: "Gmail",
    microsoft: "Outlook / M365",
    imap: "Other",
  };

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-3">
        <span className="font-medium text-sm text-white/90">
          {providerLabel[summary.provider] ?? summary.provider}
        </span>
        <span className="text-xs text-white/40">{summary.total} seeds</span>
      </div>

      {/* Stacked bar */}
      <div className="h-2 rounded-full overflow-hidden flex gap-0.5 bg-surface-border">
        {summary.total > 0 && (
          <>
            <div
              className="bg-accent-green rounded-full"
              style={{ width: `${(summary.inbox / summary.total) * 100}%` }}
            />
            <div
              className="bg-accent-purple rounded-full"
              style={{ width: `${(summary.promotions / summary.total) * 100}%` }}
            />
            <div
              className="bg-accent-orange rounded-full"
              style={{ width: `${(summary.junk / summary.total) * 100}%` }}
            />
            <div
              className="bg-accent-red rounded-full"
              style={{ width: `${(summary.spam / summary.total) * 100}%` }}
            />
          </>
        )}
      </div>

      <div className="mt-3 flex gap-3 text-xs text-white/60">
        <span>
          <span className="text-accent-green font-medium">
            {summary.inboxRate.toFixed(0)}%
          </span>{" "}
          inbox
        </span>
        {summary.spam > 0 && (
          <span>
            <span className="text-accent-red font-medium">{summary.spam}</span>{" "}
            spam
          </span>
        )}
        {summary.missing > 0 && (
          <span>
            <span className="text-white/40 font-medium">{summary.missing}</span>{" "}
            missing
          </span>
        )}
      </div>
    </div>
  );
}

function RunRow({ run }: { run: TestRun }) {
  const navigate = useNavigate();
  const statusColor: Record<string, string> = {
    complete: "text-accent-green",
    running: "text-accent-blue",
    failed: "text-accent-red",
    draft: "text-white/40",
  };

  return (
    <button
      onClick={() => navigate(`/runs/${run.id}`)}
      className="w-full flex items-center justify-between px-4 py-3 hover:bg-surface-secondary rounded-lg transition-colors text-left"
    >
      <div className="min-w-0">
        <div className="text-sm font-medium text-white/90 truncate">
          {run.subjectTemplate}
        </div>
        <div className="text-xs text-white/40 mt-0.5">
          {run.startedAt
            ? new Date(run.startedAt).toLocaleString()
            : "Not started"}
        </div>
      </div>
      <div className="flex items-center gap-3 ml-4 flex-shrink-0">
        {run.inboxRate !== undefined && (
          <span className="text-sm font-semibold text-accent-green">
            {run.inboxRate.toFixed(0)}%
          </span>
        )}
        <span className={`text-xs capitalize ${statusColor[run.status]}`}>
          {run.status}
        </span>
        <ArrowRight className="w-3.5 h-3.5 text-white/20" />
      </div>
    </button>
  );
}

export default function Dashboard() {
  const navigate = useNavigate();
  const { data: stats, isLoading } = useQuery({
    queryKey: ["dashboard"],
    queryFn: getDashboardStats,
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full text-white/40">
        Loading…
      </div>
    );
  }

  if (!stats) return null;

  const pct = (n: number) => `${(n * 100).toFixed(0)}%`;

  return (
    <div className="p-6 max-w-5xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold text-white">Dashboard</h1>
        <button onClick={() => navigate("/runs/new")} className="btn-primary">
          <Play className="w-3.5 h-3.5" />
          New Test Run
        </button>
      </div>

      {/* Top stats */}
      <div className="grid grid-cols-4 gap-3">
        <StatCard
          label="Avg Inbox Rate"
          value={pct(stats.avgInboxRate)}
          color="bg-accent-green"
          icon={CheckCircle2}
        />
        <StatCard
          label="Avg Spam Rate"
          value={pct(stats.avgSpamRate)}
          color="bg-accent-red"
          icon={XCircle}
        />
        <StatCard
          label="Avg Missing"
          value={pct(stats.avgMissingRate)}
          color="bg-surface-border"
          icon={HelpCircle}
        />
        <StatCard
          label="Total Runs"
          value={String(stats.totalRuns)}
          color="bg-accent-blue"
          icon={AlertTriangle}
        />
      </div>

      {/* Provider summary */}
      {stats.providerSummary.length > 0 && (
        <section>
          <h2 className="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">
            Provider Summary (Last Run)
          </h2>
          <div className="grid grid-cols-3 gap-3">
            {stats.providerSummary.map((s) => (
              <ProviderCard key={s.provider} summary={s} />
            ))}
          </div>
        </section>
      )}

      {/* Recent runs */}
      <section>
        <h2 className="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">
          Recent Runs
        </h2>
        {stats.recentRuns.length === 0 ? (
          <div className="card text-center py-10">
            <p className="text-white/40 text-sm mb-3">No test runs yet.</p>
            <button
              onClick={() => navigate("/runs/new")}
              className="btn-primary mx-auto"
            >
              Start your first test
            </button>
          </div>
        ) : (
          <div className="card p-0 overflow-hidden divide-y divide-surface-border">
            {stats.recentRuns.map((run) => (
              <RunRow key={run.id} run={run} />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
