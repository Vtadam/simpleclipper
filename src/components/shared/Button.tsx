import React from "react";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
}

export function Button({
  variant = "primary",
  size = "md",
  className = "",
  children,
  ...props
}: ButtonProps) {
  const base =
    "inline-flex items-center justify-center font-medium rounded-[10px] transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed";

  const variants = {
    primary:
      "bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white",
    ghost:
      "bg-transparent hover:bg-[var(--surface-2)] text-[var(--text)] border border-[var(--border)]",
    danger: "bg-red-500 hover:bg-red-600 text-white",
  };

  const sizes = {
    sm: "text-xs px-3 py-1.5",
    md: "text-sm px-4 py-2",
    lg: "text-base px-6 py-3",
  };

  return (
    <button
      className={`${base} ${variants[variant]} ${sizes[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
}
