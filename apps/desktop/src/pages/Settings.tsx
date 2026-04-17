import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2 } from "lucide-react";
import { listSendingProfiles, createSendingProfile, deleteSendingProfile } from "../api/commands";

function SendingProfileForm({ onDone }: { onDone: () => void }) {
  const qc = useQueryClient();
  const [form, setForm] = useState({
    name: "",
    fromName: "",
    fromEmail: "",
    replyTo: "",
    notes: "",
  });

  const mutation = useMutation({
    mutationFn: () =>
      createSendingProfile({
        name: form.name,
        fromName: form.fromName,
        fromEmail: form.fromEmail,
        replyTo: form.replyTo || undefined,
        notes: form.notes || undefined,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sending-profiles"] });
      onDone();
    },
  });

  const fields: { key: keyof typeof form; label: string; placeholder: string }[] = [
    { key: "name", label: "Profile name", placeholder: "e.g. Production transactional" },
    { key: "fromName", label: "From name", placeholder: "Acme Corp" },
    { key: "fromEmail", label: "From email", placeholder: "noreply@acme.com" },
    { key: "replyTo", label: "Reply-to (optional)", placeholder: "support@acme.com" },
    { key: "notes", label: "Notes (optional)", placeholder: "" },
  ];

  return (
    <div className="card space-y-3 mb-4">
      <h3 className="text-sm font-medium text-white/80">New Sending Profile</h3>
      {fields.map(({ key, label, placeholder }) => (
        <div key={key}>
          <label className="label">{label}</label>
          <input
            className="input"
            placeholder={placeholder}
            value={form[key]}
            onChange={(e) => setForm((f) => ({ ...f, [key]: e.target.value }))}
          />
        </div>
      ))}
      <div className="flex gap-2 justify-end pt-1">
        <button onClick={onDone} className="btn-secondary">Cancel</button>
        <button
          onClick={() => mutation.mutate()}
          disabled={mutation.isPending || !form.name || !form.fromEmail}
          className="btn-primary"
        >
          {mutation.isPending ? "Saving…" : "Save Profile"}
        </button>
      </div>
      {mutation.isError && (
        <p className="text-accent-red text-xs">{String(mutation.error)}</p>
      )}
    </div>
  );
}

export default function Settings() {
  const qc = useQueryClient();
  const [showNewProfile, setShowNewProfile] = useState(false);

  const { data: profiles = [] } = useQuery({
    queryKey: ["sending-profiles"],
    queryFn: listSendingProfiles,
  });

  const deleteMutation = useMutation({
    mutationFn: deleteSendingProfile,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["sending-profiles"] }),
  });

  return (
    <div className="p-6 max-w-2xl mx-auto space-y-8">
      <h1 className="text-lg font-semibold text-white">Settings</h1>

      {/* Sending profiles */}
      <section>
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xs font-medium text-white/40 uppercase tracking-wider">
            Sending Profiles
          </h2>
          <button
            onClick={() => setShowNewProfile(true)}
            className="btn-secondary text-xs"
          >
            <Plus className="w-3 h-3" />
            New Profile
          </button>
        </div>

        {showNewProfile && (
          <SendingProfileForm onDone={() => setShowNewProfile(false)} />
        )}

        <div className="space-y-2">
          {profiles.map((p) => (
            <div
              key={p.id}
              className="card flex items-center justify-between"
            >
              <div className="min-w-0">
                <div className="text-sm font-medium text-white/90">{p.name}</div>
                <div className="text-xs text-white/40 mt-0.5">
                  {p.fromName} &lt;{p.fromEmail}&gt;
                  {p.replyTo ? ` · Reply-To: ${p.replyTo}` : ""}
                </div>
                {p.notes && (
                  <div className="text-xs text-white/30 mt-0.5">{p.notes}</div>
                )}
              </div>
              <button
                onClick={() => deleteMutation.mutate(p.id)}
                className="p-1.5 rounded-md text-white/40 hover:text-accent-red hover:bg-accent-red/10 transition-colors flex-shrink-0 ml-3"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
          ))}
          {profiles.length === 0 && !showNewProfile && (
            <div className="card text-center py-8 text-white/40 text-sm">
              No sending profiles. Create one to start a test run.
            </div>
          )}
        </div>
      </section>

      {/* OAuth apps */}
      <section>
        <h2 className="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">
          OAuth App Configuration
        </h2>
        <div className="card space-y-4">
          <div>
            <label className="label">Google OAuth Client ID</label>
            <input
              className="input"
              placeholder="Set via environment variable GOOGLE_CLIENT_ID"
              disabled
            />
            <p className="text-xs text-white/30 mt-1">
              Configure in app bundle environment settings.
            </p>
          </div>
          <div>
            <label className="label">Microsoft OAuth Client ID</label>
            <input
              className="input"
              placeholder="Set via environment variable MICROSOFT_CLIENT_ID"
              disabled
            />
            <p className="text-xs text-white/30 mt-1">
              Configure in app bundle environment settings.
            </p>
          </div>
        </div>
      </section>

      {/* Polling settings */}
      <section>
        <h2 className="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">
          Polling
        </h2>
        <div className="card space-y-4">
          <div>
            <label className="label">Run timeout (minutes)</label>
            <input
              type="number"
              className="input w-32"
              defaultValue={10}
              min={1}
              max={60}
            />
            <p className="text-xs text-white/30 mt-1">
              How long to poll before marking unmatched seeds as Missing.
            </p>
          </div>
          <div>
            <label className="label">Poll interval (seconds)</label>
            <input
              type="number"
              className="input w-32"
              defaultValue={15}
              min={5}
              max={120}
            />
          </div>
        </div>
      </section>

      {/* About */}
      <section>
        <h2 className="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">
          About
        </h2>
        <div className="card text-sm text-white/50 space-y-1">
          <div className="flex justify-between">
            <span>Version</span>
            <span className="text-white/70">0.1.0</span>
          </div>
          <div className="flex justify-between">
            <span>Storage</span>
            <span className="text-white/70 font-mono text-xs">
              ~/Library/Application Support/com.mailscope.desktop
            </span>
          </div>
        </div>
      </section>
    </div>
  );
}
