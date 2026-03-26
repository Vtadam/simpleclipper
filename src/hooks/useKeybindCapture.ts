import { useState } from "react";

/**
 * Pure DOM-based keybind capture hook.
 *
 * Resolves when the user presses a valid modifier+key combination
 * (e.g. Ctrl+Shift+F9). Single modifier keys alone are not accepted —
 * at least one non-modifier key must be included.
 */
export function useKeybindCapture() {
  const [capturing, setCapturing] = useState(false);

  const capture = (): Promise<string | null> => {
    setCapturing(true);
    return new Promise((resolve) => {
      const handler = (e: KeyboardEvent) => {
        e.preventDefault();
        e.stopPropagation();

        const parts: string[] = [];
        if (e.ctrlKey) parts.push("Ctrl");
        if (e.shiftKey) parts.push("Shift");
        if (e.altKey) parts.push("Alt");

        const key = e.key;
        // Ignore bare modifier presses
        if (!["Control", "Shift", "Alt", "Meta"].includes(key)) {
          parts.push(key.length === 1 ? key.toUpperCase() : key);
        }

        // Require at least one modifier + one non-modifier key
        if (parts.length >= 2) {
          window.removeEventListener("keydown", handler, true);
          setCapturing(false);
          resolve(parts.join("+"));
        }
      };

      window.addEventListener("keydown", handler, true);
    });
  };

  return { capture, capturing };
}
