import { useQuery } from "@tanstack/react-query";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { listRuns } from "../api/commands";
import type { TestRun } from "../types";

function inboxRate(run: TestRun): number {
  if (!run.results || run.results.length === 0) return run.inboxRate ?? 0;
  const inbox = run.results.filter((r) => r.placement === "inbox").length;
  return run.results.length > 0 ? (inbox / run.results.length) * 100 : 0;
}

function spamRate(run: TestRun): number {
  if (!run.results || run.results.length === 0) return run.spamRate ?? 0;
  const spam = run.results.filter(
    (r) => r.placement === "spam" || r.placement === "junk"
  ).length;
  return run.results.length > 0 ? (spam / run.results.length) * 100 : 0;
}

export default function Trends() {
  const { data: runs = [], isLoading } = useQuery({
    queryKey: ["runs"],
    queryFn: () => listRuns(50),
  });

  const completed = runs
    .filter((r) => r.status === "complete")
    .sort(
      (a, b) =>
        new Date(a.startedAt ?? 0).getTime() -
        new Date(b.startedAt ?? 0).getTime()
    );

  const chartData = completed.map((run) => ({
    date: run.startedAt
      ? new Date(run.startedAt).toLocaleDateString("en-US", {
          month: "short",
          day: "numeric",
        })
      : "—",
    inbox: Math.round(inboxRate(run)),
    spam: Math.round(spamRate(run)),
    subject: run.subjectTemplate.slice(0, 30),
  }));

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full text-white/40">
        Loading…
      </div>
    );
  }

  return (
    <div className="p-6 max-w-5xl mx-auto space-y-6">
      <h1 className="text-lg font-semibold text-white">Trends</h1>

      {completed.length < 2 ? (
        <div className="card text-center py-16 text-white/40 text-sm">
          Complete at least 2 test runs to see trends.
        </div>
      ) : (
        <>
          {/* Inbox rate over time */}
          <section className="card">
            <h2 className="text-sm font-medium text-white/80 mb-4">
              Inbox Rate Over Time
            </h2>
            <ResponsiveContainer width="100%" height={220}>
              <LineChart
                data={chartData}
                margin={{ top: 5, right: 20, left: 0, bottom: 5 }}
              >
                <CartesianGrid strokeDasharray="3 3" stroke="#3a3a3c" />
                <XAxis
                  dataKey="date"
                  tick={{ fill: "#6b6b6d", fontSize: 11 }}
                  axisLine={{ stroke: "#3a3a3c" }}
                />
                <YAxis
                  domain={[0, 100]}
                  tick={{ fill: "#6b6b6d", fontSize: 11 }}
                  axisLine={{ stroke: "#3a3a3c" }}
                  unit="%"
                />
                <Tooltip
                  contentStyle={{
                    background: "#2c2c2e",
                    border: "1px solid #48484a",
                    borderRadius: "8px",
                    fontSize: "12px",
                    color: "#f5f5f7",
                  }}
                  formatter={(val: number) => [`${val}%`]}
                />
                <Legend
                  wrapperStyle={{ fontSize: "12px", color: "#8e8e93" }}
                />
                <Line
                  type="monotone"
                  dataKey="inbox"
                  name="Inbox %"
                  stroke="#30d158"
                  strokeWidth={2}
                  dot={{ r: 3 }}
                  activeDot={{ r: 5 }}
                />
                <Line
                  type="monotone"
                  dataKey="spam"
                  name="Spam/Junk %"
                  stroke="#ff453a"
                  strokeWidth={2}
                  dot={{ r: 3 }}
                  activeDot={{ r: 5 }}
                />
              </LineChart>
            </ResponsiveContainer>
          </section>

          {/* Run history table */}
          <section>
            <h2 className="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">
              Run History
            </h2>
            <div className="card p-0 overflow-hidden">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-surface-border">
                    <th className="px-4 py-3 text-left text-xs font-medium text-white/40">
                      Date
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-white/40">
                      Subject
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium text-white/40">
                      Inbox
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium text-white/40">
                      Spam/Junk
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium text-white/40">
                      Missing
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-surface-border/50">
                  {completed
                    .slice()
                    .reverse()
                    .map((run) => (
                      <tr key={run.id} className="hover:bg-surface-secondary/30">
                        <td className="px-4 py-3 text-white/50 text-xs">
                          {run.startedAt
                            ? new Date(run.startedAt).toLocaleDateString()
                            : "—"}
                        </td>
                        <td className="px-4 py-3 text-white/80 truncate max-w-xs">
                          {run.subjectTemplate}
                        </td>
                        <td className="px-4 py-3 text-right font-medium text-accent-green">
                          {Math.round(inboxRate(run))}%
                        </td>
                        <td className="px-4 py-3 text-right font-medium text-accent-red">
                          {Math.round(spamRate(run))}%
                        </td>
                        <td className="px-4 py-3 text-right text-white/40">
                          {Math.round((run.missingRate ?? 0) * 100)}%
                        </td>
                      </tr>
                    ))}
                </tbody>
              </table>
            </div>
          </section>
        </>
      )}
    </div>
  );
}
