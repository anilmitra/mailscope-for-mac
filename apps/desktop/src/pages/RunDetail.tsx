import { useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import {
  CheckCircle2,
  XCircle,
  HelpCircle,
  Tag,
  Clock,
  AlertTriangle,
  ChevronDown,
  ChevronRight,
  Download,
} from "lucide-react";
import { getRun, getRunResults, getRunDiagnostics, exportRunCsv } from "../api/commands";
import type { TestResult, DiagnosticIssue, Placement } from "../types";

const placementConfig: Record<
  Placement,
  { label: string; className: string; icon: React.ElementType }
> = {
  inbox: { label: "Inbox", className: "badge-inbox", icon: CheckCircle2 },
  spam: { label: "Spam", className: "badge-spam", icon: XCircle },
  junk: { label: "Junk", className: "badge-junk", icon: XCircle },
  promotions: { label: "Promotions", className: "badge-promotions", icon: Tag },
  social: { label: "Social", className: "badge-other", icon: Tag },
  updates: { label: "Updates", className: "badge-other", icon: Tag },
  missing: { label: "Missing", className: "badge-missing", icon: HelpCircle },
  other: { label: "Other", className: "badge-other", icon: Tag },
};

function PlacementBadge({ placement }: { placement: Placement }) {
  const cfg = placementConfig[placement] ?? placementConfig.other;
  return <span className={cfg.className}>{cfg.label}</span>;
}

function AuthPill({
  label,
  result,
}: {
  label: string;
  result: string | null;
}) {
  if (!result) return null;
  const pass = result.toLowerCase().includes("pass");
  return (
    <span
      className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-mono ${
        pass
          ? "bg-accent-green/10 text-accent-green"
          : "bg-accent-red/10 text-accent-red"
      }`}
    >
      {label}:{result}
    </span>
  );
}

function ResultRow({ result }: { result: TestResult }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="divide-y divide-surface-border/50">
      <button
        onClick={() => setExpanded((e) => !e)}
        className="w-full flex items-center justify-between px-4 py-3 hover:bg-surface-secondary/50 transition-colors text-left"
      >
        <div className="flex items-center gap-3 min-w-0">
          {expanded ? (
            <ChevronDown className="w-3.5 h-3.5 text-white/40 flex-shrink-0" />
          ) : (
            <ChevronRight className="w-3.5 h-3.5 text-white/40 flex-shrink-0" />
          )}
          <div className="min-w-0">
            <div className="text-sm text-white/90 truncate">
              {result.account?.seedEmail ?? `Account #${result.connectedAccountId}`}
            </div>
            <div className="flex items-center gap-2 mt-1">
              <AuthPill label="SPF" result={result.spfResult} />
              <AuthPill label="DKIM" result={result.dkimResult} />
              <AuthPill label="DMARC" result={result.dmarcResult} />
            </div>
          </div>
        </div>
        <div className="flex items-center gap-3 ml-4 flex-shrink-0">
          {result.deliveryLatencyMs != null && (
            <span className="flex items-center gap-1 text-xs text-white/40">
              <Clock className="w-3 h-3" />
              {(result.deliveryLatencyMs / 1000).toFixed(1)}s
            </span>
          )}
          <PlacementBadge placement={result.placement} />
        </div>
      </button>

      {expanded && result.headersJson && (
        <div className="px-4 py-3 bg-surface/50">
          <div className="text-xs text-white/40 uppercase tracking-wider mb-2">
            Headers
          </div>
          <div className="font-mono text-xs space-y-1 max-h-64 overflow-y-auto">
            {Object.entries(result.headersJson).map(([k, v]) => (
              <div key={k} className="flex gap-2">
                <span className="text-accent-blue/70 flex-shrink-0">{k}:</span>
                <span className="text-white/60 break-all">{v}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function DiagnosticCard({ issue }: { issue: DiagnosticIssue }) {
  const colors: Record<string, string> = {
    critical: "border-accent-red/40 bg-accent-red/5",
    error: "border-accent-orange/40 bg-accent-orange/5",
    warning: "border-accent-yellow/40 bg-accent-yellow/5",
    info: "border-surface-border bg-surface/50",
  };

  const iconColors: Record<string, string> = {
    critical: "text-accent-red",
    error: "text-accent-orange",
    warning: "text-accent-yellow",
    info: "text-white/40",
  };

  return (
    <div className={`card border ${colors[issue.severity]}`}>
      <div className="flex items-start gap-3">
        <AlertTriangle
          className={`w-4 h-4 flex-shrink-0 mt-0.5 ${iconColors[issue.severity]}`}
        />
        <div className="min-w-0">
          <div className="text-sm font-medium text-white/90">{issue.title}</div>
          <div className="text-xs text-white/50 mt-1">{issue.details}</div>
          {issue.recommendation && (
            <div className="mt-2 text-xs text-accent-blue/80 bg-accent-blue/5 border border-accent-blue/20 rounded-md px-3 py-2">
              <strong>Fix:</strong> {issue.recommendation}
            </div>
          )}
        </div>
        <span className="text-xs text-white/30 flex-shrink-0 uppercase">
          {issue.checkType}
        </span>
      </div>
    </div>
  );
}

export default function RunDetail() {
  const { id } = useParams<{ id: string }>();
  const runId = parseInt(id ?? "0");
  const [tab, setTab] = useState<"results" | "diagnostics">("results");

  const { data: run } = useQuery({
    queryKey: ["run", runId],
    queryFn: () => getRun(runId),
    refetchInterval: (q) =>
      q.state.data?.status === "running" ? 5000 : false,
  });

  const { data: results = [] } = useQuery({
    queryKey: ["run-results", runId],
    queryFn: () => getRunResults(runId),
    refetchInterval: run?.status === "running" ? 5000 : false,
    enabled: !!run,
  });

  const { data: diagnostics = [] } = useQuery({
    queryKey: ["run-diagnostics", runId],
    queryFn: () => getRunDiagnostics(runId),
    enabled: run?.status === "complete",
  });

  if (!run) {
    return (
      <div className="flex items-center justify-center h-full text-white/40">
        Loading…
      </div>
    );
  }

  const counts = results.reduce(
    (acc, r) => {
      acc[r.placement] = (acc[r.placement] ?? 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  const handleExport = async () => {
    const csv = await exportRunCsv(runId);
    const blob = new Blob([csv], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `mailscope-run-${run.runUuid}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="p-6 max-w-4xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div className="min-w-0">
          <h1 className="text-lg font-semibold text-white truncate">
            {run.subjectTemplate}
          </h1>
          <div className="flex items-center gap-3 mt-1 text-xs text-white/40">
            <span
              className={`capitalize ${
                run.status === "running"
                  ? "text-accent-blue"
                  : run.status === "complete"
                    ? "text-accent-green"
                    : run.status === "failed"
                      ? "text-accent-red"
                      : "text-white/40"
              }`}
            >
              {run.status}
            </span>
            {run.startedAt && (
              <span>{new Date(run.startedAt).toLocaleString()}</span>
            )}
            <code className="font-mono text-white/25">{run.subjectToken}</code>
          </div>
        </div>
        <button onClick={handleExport} className="btn-secondary flex-shrink-0">
          <Download className="w-3.5 h-3.5" />
          Export CSV
        </button>
      </div>

      {/* Summary bar */}
      {results.length > 0 && (
        <div className="grid grid-cols-4 gap-3">
          {(["inbox", "spam", "junk", "missing"] as Placement[]).map((p) => (
            <div key={p} className="card text-center">
              <div className="text-2xl font-semibold text-white">
                {counts[p] ?? 0}
              </div>
              <div className="text-xs text-white/40 capitalize mt-1">{p}</div>
            </div>
          ))}
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-1 border-b border-surface-border">
        {(["results", "diagnostics"] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px capitalize ${
              tab === t
                ? "border-accent-blue text-accent-blue"
                : "border-transparent text-white/40 hover:text-white/70"
            }`}
          >
            {t}
            {t === "diagnostics" && diagnostics.length > 0 && (
              <span className="ml-1.5 px-1.5 py-0.5 rounded-full text-xs bg-accent-orange/20 text-accent-orange">
                {diagnostics.length}
              </span>
            )}
          </button>
        ))}
      </div>

      {tab === "results" && (
        <div className="card p-0 overflow-hidden divide-y divide-surface-border">
          {results.length === 0 ? (
            <div className="text-center py-10 text-white/40 text-sm">
              {run.status === "running"
                ? "Polling mailboxes…"
                : "No results yet."}
            </div>
          ) : (
            results.map((r) => <ResultRow key={r.id} result={r} />)
          )}
        </div>
      )}

      {tab === "diagnostics" && (
        <div className="space-y-3">
          {diagnostics.length === 0 ? (
            <div className="card text-center py-10 text-white/40 text-sm">
              {run.status === "complete"
                ? "No issues detected."
                : "Diagnostics available after run completes."}
            </div>
          ) : (
            diagnostics.map((d) => <DiagnosticCard key={d.id} issue={d} />)
          )}
        </div>
      )}
    </div>
  );
}
