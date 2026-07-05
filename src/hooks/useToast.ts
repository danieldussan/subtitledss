import { useState, useCallback, useRef } from "react";

export type ToastType = "success" | "error" | "warning" | "info";

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}

let toastId = 0;

export function useToast() {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const timersRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const removeToast = useCallback((id: string) => {
    const timer = timersRef.current.get(id);
    if (timer) {
      clearTimeout(timer);
      timersRef.current.delete(id);
    }
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const addToast = useCallback(
    (type: ToastType, message: string, duration = 5000) => {
      const id = `toast-${++toastId}`;
      const toast: Toast = { id, type, message, duration };

      setToasts((prev) => {
        const next = [...prev, toast];
        // Keep max 3 visible
        return next.length > 3 ? next.slice(-3) : next;
      });

      if (duration > 0) {
        const timer = setTimeout(() => removeToast(id), duration);
        timersRef.current.set(id, timer);
      }

      return id;
    },
    [removeToast],
  );

  const success = useCallback(
    (message: string, duration?: number) => addToast("success", message, duration),
    [addToast],
  );

  const error = useCallback(
    (message: string, duration?: number) => addToast("error", message, duration ?? 7000),
    [addToast],
  );

  const warning = useCallback(
    (message: string, duration?: number) => addToast("warning", message, duration),
    [addToast],
  );

  const info = useCallback(
    (message: string, duration?: number) => addToast("info", message, duration),
    [addToast],
  );

  return { toasts, addToast, removeToast, success, error, warning, info };
}
