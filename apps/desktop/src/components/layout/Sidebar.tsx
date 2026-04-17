import { NavLink, useNavigate } from "react-router-dom";
import {
  LayoutDashboard,
  Mailbox,
  Play,
  TrendingUp,
  Settings,
  FlaskConical,
} from "lucide-react";

const navItems = [
  { to: "/dashboard", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/mailboxes", icon: Mailbox, label: "Seed Mailboxes" },
  { to: "/trends", icon: TrendingUp, label: "Trends" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export default function Sidebar() {
  const navigate = useNavigate();

  return (
    <aside className="w-52 flex-shrink-0 bg-surface/80 backdrop-blur border-r border-surface-border flex flex-col select-none">
      {/* App title */}
      <div className="px-4 pt-8 pb-5 flex items-center gap-2">
        <FlaskConical className="w-5 h-5 text-accent-blue" />
        <span className="font-semibold text-sm text-white">MailScope</span>
      </div>

      {/* New Test Run button */}
      <div className="px-3 mb-4">
        <button
          onClick={() => navigate("/runs/new")}
          className="btn-primary w-full justify-center text-xs"
        >
          <Play className="w-3.5 h-3.5" />
          New Test Run
        </button>
      </div>

      {/* Nav */}
      <nav className="flex-1 px-2 space-y-0.5">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              `flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm transition-colors ${
                isActive
                  ? "bg-accent-blue/20 text-accent-blue font-medium"
                  : "text-white/60 hover:text-white/90 hover:bg-surface-secondary"
              }`
            }
          >
            <Icon className="w-4 h-4 flex-shrink-0" />
            {label}
          </NavLink>
        ))}
      </nav>

      {/* Footer */}
      <div className="px-4 pb-4 text-xs text-white/25">v0.1.0</div>
    </aside>
  );
}
