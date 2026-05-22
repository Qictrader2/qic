/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./app/**/*.{ts,tsx}", "./src/**/*.{ts,tsx}", "./components/**/*.{ts,tsx}"],
  presets: [require("nativewind/preset")],
  theme: {
    extend: {
      colors: {
        brand: "#00A3F6",
        "brand-light": "#38BDF8",
        "brand-dark": "#0284C7",
        "brand-bg": "rgba(0,163,246,0.1)",
        success: "#10B981",
        "success-bg": "rgba(16,185,129,0.1)",
        warning: "#F59E0B",
        "warning-bg": "rgba(245,158,11,0.1)",
        error: "#EF4444",
        "error-bg": "rgba(239,68,68,0.1)",
        info: "#3B82F6",
        "info-bg": "rgba(59,130,246,0.1)",
        btc: "#F7931A",
        eth: "#627EEA",
        sol: "#9945FF",
        usdt: "#26A17B",
        background: {
          DEFAULT: "#FFFFFF",
          dark: "#000000",
        },
        surface: {
          DEFAULT: "#FFFFFF",
          dark: "#191F2A",
        },
        foreground: {
          DEFAULT: "#0F172A",
          dark: "#F8FAFC",
        },
        muted: {
          DEFAULT: "#475569",
          dark: "#94A3B8",
        },
        border: {
          DEFAULT: "#E2E8F0",
          dark: "rgba(255,255,255,0.1)",
        },
      },
      fontFamily: {
        sans: ["Poppins_400Regular", "sans-serif"],
        medium: ["Poppins_500Medium", "sans-serif"],
        semibold: ["Poppins_600SemiBold", "sans-serif"],
        bold: ["Poppins_700Bold", "sans-serif"],
      },
      borderRadius: {
        DEFAULT: "10px",
        sm: "6px",
        md: "6px",
        lg: "8px",
        xl: "12px",
        "2xl": "16px",
        "3xl": "24px",
      },
    },
  },
  plugins: [],
}
