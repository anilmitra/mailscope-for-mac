import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, RefreshCw, CheckCircle2, AlertCircle, Mail } from "lucide-react";
import {
  listAccounts,
  removeAccount,
  checkAccountHealth,
  addImapAccount,
  listSeedGroups,
  createSeedGroup,
} from "../api/commands";
import type { ConnectedAccount, AuthType } from "../types";

const providerLabel: Record<string, string> = {
  gmail: "Gmail",
  microsoft: "Microsoft / Outlook",
  imap: "IMAP",
};

function AccountRow({
  account,
  onRemove,
  onCheck,
}: {
  account: ConnectedAccount;
  onRemove: (id: number) => void;
  onCheck: (id: number) => void;
}) {
  return (
    <div className="flex items-center justify-between px-4 py-3">
      <div className="flex items-center gap-3 min-w-0">
        <Mail className="w-4 h-4 text-white/40 flex-shrink-0" />
        <div className="min-w-0">
          <div className="text-sm font-medium text-white/90 truncate">
            {account.displayName}
          </div>
          <div className="text-xs text-white/40 truncate">{account.seedEmail}</div>
        </div>
      </div>
      <div className="flex items-center gap-3 ml-4 flex-shrink-0">
        <span className="text-xs text-white/40">{providerLabel[account.provider]}</span>
        {account.status === "active" ? (
          <CheckCircle2 className="w-4 h-4 text-accent-green" />
        ) : (
          <AlertCircle className="w-4 h-4 text-accent-orange" />
        )}
        <button
          onClick={() => onCheck(account.id)}
          className="p-1.5 rounded-md text-white/40 hover:text-white hover:bg-surface-tertiary transition-colors"
          title="Check health"
        >
          <RefreshCw className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={() => onRemove(account.id)}
          className="p-1.5 rounded-md text-white/40 hover:text-accent-red hover:bg-accent-red/10 transition-colors"
          title="Remove"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}

function AddImapModal({ onClose }: { onClose: () => void }) {
  const qc = useQueryClient();
  const [form, setForm] = useState({
    displayName: "",
    seedEmail: "",
    host: "",
    port: "993",
    authType: "app_password" as AuthType,
    password: "",
  });
  const mutation = useMutation({
    mutationFn: () => addImapAccount({ ...form, port: parseInt(form.port) }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["accounts"] });
      onClose();
    },
  });

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="card w-96 space-y-4">
        <h2 className="font-semibold text-white">Add IMAP Mailbox</h2>
        {(["displayName", "seedEmail", "host", "port"] as const).map((field) => (
          <div key={field}>
            <label className="label">{field}</label>
            <input
              className="input"
              value={form[field]}
              onChange={(e) => setForm((f) => ({ ...f, [field]: e.target.value }))}
            />
          </div>
        ))}
        <div>
          <label className="label">Password</label>
          <input
            type="password"
            className="input"
            value={form.password}
            onChange={(e) => setForm((f) => ({ ...f, password: e.target.value }))}
          />
        </div>
        <div className="flex gap-2 justify-end pt-2">
          <button onClick={onClose} className="btn-secondary">Cancel</button>
          <button
            onClick={() => mutation.mutate()}
            disabled={mutation.isPending}
            className="btn-primary"
          >
            {mutation.isPending ? "Connecting…" : "Connect"}
          </button>
        </div>
        {mutation.isError && (
          <p className="text-accent-red text-xs">{String(mutation.error)}</p>
        )}
      </div>
    </div>
  );
}

export default function SeedMailboxes() {
  const qc = useQueryClient();
  const [showAddImap, setShowAddImap] = useState(false);
  const [showAddGroup, setShowAddGroup] = useState(false);
  const [newGroupName, setNewGroupName] = useState("");

  const { data: accounts = [] } = useQuery({
    queryKey: ["accounts"],
    queryFn: listAccounts,
  });

  const { data: groups = [] } = useQuery({
    queryKey: ["seed-groups"],
    queryFn: listSeedGroups,
  });

  const removeMutation = useMutation({
    mutationFn: removeAccount,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["accounts"] }),
  });

  const checkMutation = useMutation({
    mutationFn: checkAccountHealth,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["accounts"] }),
  });

  const createGroupMutation = useMutation({
    mutationFn: () => createSeedGroup(newGroupName),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["seed-groups"] });
      setNewGroupName("");
      setShowAddGroup(false);
    },
  });

  return (
    <div className="p-6 max-w-4xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold text-white">Seed Mailboxes</h1>
        <div className="flex gap-2">
          <button
            onClick={() => setShowAddImap(true)}
            className="btn-secondary"
          >
            <Plus className="w-3.5 h-3.5" />
            Add IMAP
          </button>
          <button className="btn-primary">
            <Plus className="w-3.5 h-3.5" />
            Connect Gmail
          </button>
          <button className="btn-primary">
            <Plus className="w-3.5 h-3.5" />
            Connect Microsoft
          </button>
        </div>
      </div>

      {/* Accounts list */}
      <section>
        <h2 className="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">
          Connected Accounts ({accounts.length})
        </h2>
        {accounts.length === 0 ? (
          <div className="card text-center py-8 text-white/40 text-sm">
            No mailboxes connected yet.
          </div>
        ) : (
          <div className="card p-0 overflow-hidden divide-y divide-surface-border">
            {accounts.map((a) => (
              <AccountRow
                key={a.id}
                account={a}
                onRemove={(id) => removeMutation.mutate(id)}
                onCheck={(id) => checkMutation.mutate(id)}
              />
            ))}
          </div>
        )}
      </section>

      {/* Seed groups */}
      <section>
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xs font-medium text-white/40 uppercase tracking-wider">
            Seed Groups ({groups.length})
          </h2>
          <button
            onClick={() => setShowAddGroup(true)}
            className="btn-secondary text-xs"
          >
            <Plus className="w-3 h-3" />
            New Group
          </button>
        </div>

        {showAddGroup && (
          <div className="card mb-3 flex gap-2">
            <input
              className="input flex-1"
              placeholder="Group name"
              value={newGroupName}
              onChange={(e) => setNewGroupName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && createGroupMutation.mutate()}
            />
            <button onClick={() => createGroupMutation.mutate()} className="btn-primary">
              Create
            </button>
            <button onClick={() => setShowAddGroup(false)} className="btn-secondary">
              Cancel
            </button>
          </div>
        )}

        <div className="space-y-2">
          {groups.map((g) => (
            <div key={g.id} className="card flex items-center justify-between">
              <div>
                <div className="text-sm font-medium text-white/90">{g.name}</div>
                {g.description && (
                  <div className="text-xs text-white/40 mt-0.5">{g.description}</div>
                )}
              </div>
              <span className="text-xs text-white/40">
                {g.memberCount ?? 0} accounts
              </span>
            </div>
          ))}
          {groups.length === 0 && !showAddGroup && (
            <div className="card text-center py-8 text-white/40 text-sm">
              No seed groups yet. Create one to organize your mailboxes.
            </div>
          )}
        </div>
      </section>

      {showAddImap && <AddImapModal onClose={() => setShowAddImap(false)} />}
    </div>
  );
}
