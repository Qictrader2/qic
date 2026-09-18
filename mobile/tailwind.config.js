/** @type {import('tailwindcss').Config} */
// All color tokens mirror Frontend/src/app/globals.css exactly.
// Brand blue #00A3F6, exact light + dark palettes from web.
module.exports = {
  content: [
    "./app/**/*.{js,jsx,ts,tsx}",
    "./src/**/*.{js,jsx,ts,tsx}",
  ],
  presets: [require("nativewind/preset")],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // ===== BRAND (consistent across themes) =====
        brand: {
          DEFAULT: "#00A3F6",          // brand-blue
          blue: "#00A3F6",
          "blue-light": "#38BDF8",
          "blue-dark": "#0284C7",
          "blue-bg": "rgba(0, 163, 246, 0.1)",
          green: "#10B981",
          "green-dark": "#059669",
          red: "#EF4444",
          "red-dark": "#DC2626",
        },
        // Mobile-friendly aliases
        "brand-bg": "rgba(0, 163, 246, 0.1)",

        // ===== LIGHT MODE =====
        background: "#FFFFFF",
        "background-secondary": "#FFFFFF",
        "background-gray": "#F6F6F6",
        "background-gray-light": "#EFEFEF",
        foreground: "#0F172A",
        card: "#FFFFFF",
        "card-foreground": "#0F172A",
        popover: "#FFFFFF",
        "popover-foreground": "#0F172A",
        primary: "#0F172A",
        "primary-foreground": "#F8FAFC",
        secondary: "#F1F5F9",
        "secondary-foreground": "#0F172A",
        muted: "#64748B",                     // text-secondary
        "muted-foreground": "#475569",
        accent: "#F1F5F9",
        "accent-foreground": "#0F172A",
        border: "#E2E8F0",
        input: "#E2E8F0",

        // Surfaces
        surface: "#F8FAFC",
        "surface-hover": "#F1F5F9",
        "hero-bg": "#F8FAFC",

        // Semantic
        success: "#10B981",
        "success-bg": "rgba(16, 185, 129, 0.1)",
        warning: "#F59E0B",
        "warning-bg": "rgba(245, 158, 11, 0.1)",
        error: "#EF4444",
        "error-bg": "rgba(239, 68, 68, 0.1)",
        info: "#3B82F6",
        "info-bg": "rgba(59, 130, 246, 0.1)",

        // ===== DARK MODE (suffixed -dark for explicit RN usage) =====
        "background-dark": "#000000",
        "background-secondary-dark": "#111111",
        "background-gray-dark": "#040607",
        "foreground-dark": "#F8FAFC",
        "card-dark": "#191F2A",
        "surface-dark": "#1E293B",
        "surface-hover-dark": "#334155",
        "border-dark": "rgba(255, 255, 255, 0.1)",
        "muted-dark": "#94A3B8",
        "muted-foreground-dark": "#94A3B8",
        "hero-bg-dark": "#111111",
      },
      fontFamily: {
        sans: ["Poppins_400Regular"],
        medium: ["Poppins_500Medium"],
        semibold: ["Poppins_600SemiBold"],
        bold: ["Poppins_700Bold"],
      },
      borderRadius: {
        sm: "6px",   // calc(0.625rem - 4px)
        DEFAULT: "8px",
        md: "8px",   // calc(0.625rem - 2px)
        lg: "10px",  // 0.625rem
        xl: "14px",  // calc(0.625rem + 4px)
        "2xl": "20px",
      },
    },
  },
  plugins: [],
}
