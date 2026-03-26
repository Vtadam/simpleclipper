import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter", "-apple-system", "BlinkMacSystemFont", "sans-serif"],
      },
      colors: {
        accent: "#3B82F6",
      },
      borderRadius: {
        DEFAULT: "10px",
        sm: "6px",
      },
    },
  },
  plugins: [],
} satisfies Config;
