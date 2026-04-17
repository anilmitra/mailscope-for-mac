/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // Remapped to CSS vars so dark/light theme works without touching components.
        // In dark mode --c-white ≈ #f5f5f7; in light mode it swaps to #1c1c1e so
        // text-white/50, text-white/90, etc. all work correctly in both themes.
        white: "rgb(var(--c-white) / <alpha-value>)",
        surface: {
          DEFAULT: "rgb(var(--c-surface) / <alpha-value>)",
          secondary: "rgb(var(--c-surface-secondary) / <alpha-value>)",
          tertiary: "rgb(var(--c-surface-tertiary) / <alpha-value>)",
          border: "rgb(var(--c-surface-border) / <alpha-value>)",
        },
        accent: {
          blue: "rgb(var(--c-accent-blue) / <alpha-value>)",
          green: "rgb(var(--c-accent-green) / <alpha-value>)",
          orange: "rgb(var(--c-accent-orange) / <alpha-value>)",
          red: "rgb(var(--c-accent-red) / <alpha-value>)",
          purple: "rgb(var(--c-accent-purple) / <alpha-value>)",
          yellow: "rgb(var(--c-accent-yellow) / <alpha-value>)",
        },
      },
      fontFamily: {
        sans: [
          "-apple-system",
          "BlinkMacSystemFont",
          "SF Pro Text",
          "Segoe UI",
          "system-ui",
          "sans-serif",
        ],
        mono: ["SF Mono", "JetBrains Mono", "Fira Code", "monospace"],
      },
    },
  },
  plugins: [],
};
