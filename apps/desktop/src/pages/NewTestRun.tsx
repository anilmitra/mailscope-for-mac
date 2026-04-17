import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Copy, CheckCheck, ChevronRight, ChevronLeft, Play } from "lucide-react";
import {
  listSendingProfiles,
  listSeedGroups,
  generateRunToken,
  createRun,
  startRun,
} from "../api/commands";
import type { RunToken } from "../types";

const STEPS = ["Profile", "Seeds", "Content", "Review"] as const;
type Step = (typeof STEPS)[number];

function StepIndicator({ current }: { current: Step }) {
  const idx = STEPS.indexOf(current);
  return (
    <div className="flex items-center gap-2 mb-8">
      {STEPS.map((s, i) => (
        <div key={s} className="flex items-center gap-2">
          <div
            className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium ${
              i < idx
                ? "bg-accent-green text-white"
                : i === idx
                  ? "bg-accent-blue text-white"
                  : "bg-surface-tertiary text-white/40"
            }`}
          >
            {i + 1}
          </div>
          <span
            className={`text-sm ${i === idx ? "text-white" : "text-white/40"}`}
          >
            {s}
          </span>
          {i < STEPS.length - 1 && (
            <ChevronRight className="w-3.5 h-3.5 text-white/20" />
          )}
        </div>
      ))}
    </div>
  );
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={copy}
      className="p-1.5 rounded-md text-white/40 hover:text-white hover:bg-surface-tertiary transition-colors"
    >
      {copied ? (
        <CheckCheck className="w-3.5 h-3.5 text-accent-green" />
      ) : (
        <Copy className="w-3.5 h-3.5" />
      )}
    </button>
  );
}

export default function NewTestRun() {
  const navigate = useNavigate();
  const [step, setStep] = useState<Step>("Profile");
  const [selectedProfileId, setSelectedProfileId] = useState<number | null>(null);
  const [selectedGroupId, setSelectedGroupId] = useState<number | null>(null);
  const [subject, setSubject] = useState("");
  const [bodyText, setBodyText] = useState(
    "Hi,\n\nThis is a placement test message.\n\nPlease ignore.\n\n--\nSent via MailScope"
  );
  const [token, setToken] = useState<RunToken | null>(null);

  const { data: profiles = [] } = useQuery({
    queryKey: ["sending-profiles"],
    queryFn: listSendingProfiles,
  });

  const { data: groups = [] } = useQuery({
    queryKey: ["seed-groups"],
    queryFn: listSeedGroups,
  });

  useEffect(() => {
    if (selectedProfileId && subject) {
      generateRunToken(selectedProfileId, subject).then(setToken).catch(() => {});
    }
  }, [selectedProfileId, subject]);

  const createMutation = useMutation({
    mutationFn: async () => {
      if (!selectedProfileId || !selectedGroupId || !token) {
        throw new Error("Missing required fields");
      }
      const run = await createRun({
        sendingProfileId: selectedProfileId,
        seedGroupId: selectedGroupId,
        subjectTemplate: subject,
        bodyText,
        subjectToken: token.subjectToken,
        bodyToken: token.bodyToken,
      });
      await startRun(run.id);
      return run;
    },
    onSuccess: (run) => navigate(`/runs/${run.id}`),
  });

  const canNext =
    step === "Profile"
      ? selectedProfileId !== null
      : step === "Seeds"
        ? selectedGroupId !== null
        : step === "Content"
          ? subject.trim().length > 0
          : true;

  const next = () => {
    const idx = STEPS.indexOf(step);
    if (idx < STEPS.length - 1) setStep(STEPS[idx + 1]);
  };

  const back = () => {
    const idx = STEPS.indexOf(step);
    if (idx > 0) setStep(STEPS[idx - 1]);
  };

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <h1 className="text-lg font-semibold text-white mb-6">New Test Run</h1>
      <StepIndicator current={step} />

      {step === "Profile" && (
        <div className="space-y-3">
          <h2 className="text-sm font-medium text-white/80 mb-4">
            Select a sending profile
          </h2>
          {profiles.length === 0 && (
            <div className="card text-center py-6 text-white/40 text-sm">
              No sending profiles yet.{" "}
              <button
                onClick={() => navigate("/settings")}
                className="text-accent-blue underline"
              >
                Create one in Settings.
              </button>
            </div>
          )}
          {profiles.map((p) => (
            <button
              key={p.id}
              onClick={() => setSelectedProfileId(p.id)}
              className={`w-full text-left card hover:border-accent-blue transition-colors ${
                selectedProfileId === p.id ? "border-accent-blue" : ""
              }`}
            >
              <div className="font-medium text-sm text-white/90">{p.name}</div>
              <div className="text-xs text-white/40 mt-1">
                {p.fromName} &lt;{p.fromEmail}&gt;
              </div>
            </button>
          ))}
        </div>
      )}

      {step === "Seeds" && (
        <div className="space-y-3">
          <h2 className="text-sm font-medium text-white/80 mb-4">
            Choose a seed list
          </h2>
          {groups.length === 0 && (
            <div className="card text-center py-6 text-white/40 text-sm">
              No seed groups yet.{" "}
              <button
                onClick={() => navigate("/mailboxes")}
                className="text-accent-blue underline"
              >
                Add mailboxes first.
              </button>
            </div>
          )}
          {groups.map((g) => (
            <button
              key={g.id}
              onClick={() => setSelectedGroupId(g.id)}
              className={`w-full text-left card hover:border-accent-blue transition-colors ${
                selectedGroupId === g.id ? "border-accent-blue" : ""
              }`}
            >
              <div className="font-medium text-sm text-white/90">{g.name}</div>
              <div className="text-xs text-white/40 mt-1">
                {g.memberCount ?? 0} seed accounts
                {g.description ? ` · ${g.description}` : ""}
              </div>
            </button>
          ))}
        </div>
      )}

      {step === "Content" && (
        <div className="space-y-4">
          <h2 className="text-sm font-medium text-white/80 mb-2">
            Define your test message
          </h2>
          <div>
            <label className="label">Subject line</label>
            <input
              className="input"
              placeholder="e.g. Your weekly digest"
              value={subject}
              onChange={(e) => setSubject(e.target.value)}
            />
          </div>
          <div>
            <label className="label">Plain text body</label>
            <textarea
              className="input font-mono resize-none"
              rows={6}
              value={bodyText}
              onChange={(e) => setBodyText(e.target.value)}
            />
          </div>
          {token && (
            <div className="card bg-surface space-y-3">
              <p className="text-xs text-white/50 font-medium uppercase tracking-wider">
                Generated tokens (include these in your send)
              </p>
              <div className="space-y-2">
                <div>
                  <div className="text-xs text-white/40 mb-1">
                    Subject with token
                  </div>
                  <div className="flex items-center gap-2 bg-surface-tertiary rounded-lg px-3 py-2">
                    <code className="text-xs text-white/80 flex-1 font-mono">
                      {token.suggestedSubject}
                    </code>
                    <CopyButton text={token.suggestedSubject} />
                  </div>
                </div>
                <div>
                  <div className="text-xs text-white/40 mb-1">
                    X-MailScope-Token header value
                  </div>
                  <div className="flex items-center gap-2 bg-surface-tertiary rounded-lg px-3 py-2">
                    <code className="text-xs text-white/80 flex-1 font-mono">
                      {token.subjectToken}
                    </code>
                    <CopyButton text={token.subjectToken} />
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {step === "Review" && (
        <div className="space-y-4">
          <h2 className="text-sm font-medium text-white/80 mb-2">
            Review and start
          </h2>
          <div className="card space-y-3">
            <div className="flex justify-between text-sm">
              <span className="text-white/50">Sending profile</span>
              <span className="text-white/90">
                {profiles.find((p) => p.id === selectedProfileId)?.name ?? "—"}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-white/50">Seed group</span>
              <span className="text-white/90">
                {groups.find((g) => g.id === selectedGroupId)?.name ?? "—"}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-white/50">Subject</span>
              <span className="text-white/90 truncate max-w-xs text-right">
                {subject}
              </span>
            </div>
            {token && (
              <div className="flex justify-between text-sm">
                <span className="text-white/50">Token</span>
                <code className="text-white/70 font-mono text-xs">
                  {token.subjectToken}
                </code>
              </div>
            )}
          </div>
          <div className="card bg-surface-secondary border-accent-blue/30">
            <p className="text-xs text-white/60 leading-relaxed">
              <strong className="text-white/80">Manual send workflow:</strong>{" "}
              After clicking Start, send your test email from your normal sending
              tool using the subject and token above. MailScope will begin polling
              your seed mailboxes automatically.
            </p>
          </div>
          {createMutation.isError && (
            <p className="text-accent-red text-xs">{String(createMutation.error)}</p>
          )}
        </div>
      )}

      {/* Navigation */}
      <div className="flex justify-between mt-8">
        {step !== "Profile" ? (
          <button onClick={back} className="btn-secondary">
            <ChevronLeft className="w-3.5 h-3.5" />
            Back
          </button>
        ) : (
          <div />
        )}
        {step !== "Review" ? (
          <button onClick={next} disabled={!canNext} className="btn-primary">
            Next
            <ChevronRight className="w-3.5 h-3.5" />
          </button>
        ) : (
          <button
            onClick={() => createMutation.mutate()}
            disabled={createMutation.isPending || !canNext}
            className="btn-primary"
          >
            <Play className="w-3.5 h-3.5" />
            {createMutation.isPending ? "Starting…" : "Start Run"}
          </button>
        )}
      </div>
    </div>
  );
}
