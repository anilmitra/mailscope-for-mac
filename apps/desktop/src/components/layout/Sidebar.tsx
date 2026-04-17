import { NavLink, useNavigate } from "react-router-dom";
import {
  LayoutDashboard,
  Mailbox,
  Play,
  TrendingUp,
  Settings,
  FlaskConical,
  Moon,
  Sun,
  Monitor,
} from "lucide-react";
import { useTheme, type Theme } from "../../hooks/useTheme";

const navItems = [
  { to: "/dashboard", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/mailboxes", icon: Mailbox, label: "Seed Mailboxes" },
  { to: "/trends", icon: TrendingUp, label: "Trends" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

const themeIcon: Record<Theme, typeof Moon> = {
  dark: Moon,
  light: Sun,
  system: Monitor,
};

const nextTheme: Record<Theme, Theme> = {
  dark: "light",
  light: "system",
  system: "dark",
};

const themeLabel: Record<Theme, string> = {
  dark: "Dark",
  light: "Light",
  system: "System",
};

export default function Sidebar() {
  const navigate = useNavigate();
  const { theme, setTheme } = useTheme();
  const ThemeIcon = themeIcon[theme];

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

      {/* Footer — theme toggle + version */}
      <div className="px-3 pb-4 flex items-center justify-between">
        <span className="text-xs text-white/25">v0.1.0</span>
        <button
          onClick={() => setTheme(nextTheme[theme])}
          title={`Theme: ${themeLabel[theme]} — click to switch`}
          className="flex items-center gap-1.5 px-2 py-1 rounded-md text-xs
            text-white/40 hover:text-white/70 hover:bg-surface-secondary
            transition-colors"
        >
          <ThemeIcon className="w-3.5 h-3.5" />
          <span>{themeLabel[theme]}</span>
        </button>
      </div>
    </aside>
  );
}
