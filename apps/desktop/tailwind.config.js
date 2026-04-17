/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: "#1c1c1e",
          secondary: "#2c2c2e",
          tertiary: "#3a3a3c",
          border: "#48484a",
        },
        accent: {
          blue: "#0a84ff",
          green: "#30d158",
          orange: "#ff9f0a",
          red: "#ff453a",
          purple: "#bf5af2",
          yellow: "#ffd60a",
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
