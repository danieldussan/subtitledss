import { motion, AnimatePresence } from "framer-motion";
import { CheckCircle, XCircle, AlertTriangle, Info, X } from "lucide-react";
import { Toast as ToastType } from "../../hooks/useToast";

interface ToastContainerProps {
  toasts: ToastType[];
  onRemove: (id: string) => void;
}

const icons = {
  success: CheckCircle,
  error: XCircle,
  warning: AlertTriangle,
  info: Info,
};

const styles = {
  success: "border-success/30 bg-success-subtle",
  error: "border-danger/30 bg-danger-subtle",
  warning: "border-warning/30 bg-warning-subtle",
  info: "border-accent/30 bg-accent-subtle",
};

const iconColors = {
  success: "text-success",
  error: "text-danger",
  warning: "text-warning",
  info: "text-accent",
};

export function ToastContainer({ toasts, onRemove }: ToastContainerProps) {
  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
      <AnimatePresence mode="popLayout">
        {toasts.map((toast) => {
          const Icon = icons[toast.type];
          return (
            <motion.div
              key={toast.id}
              layout
              initial={{ opacity: 0, x: 50, scale: 0.95 }}
              animate={{ opacity: 1, x: 0, scale: 1 }}
              exit={{ opacity: 0, x: 50, scale: 0.95 }}
              transition={{ duration: 0.2, ease: "easeOut" }}
              className={`pointer-events-auto flex items-start gap-3 px-4 py-3 rounded-lg border backdrop-blur-sm shadow-lg max-w-sm ${styles[toast.type]}`}
            >
              <Icon size={16} className={`mt-0.5 flex-shrink-0 ${iconColors[toast.type]}`} />
              <span className="text-[13px] text-text-primary flex-1 leading-relaxed">
                {toast.message}
              </span>
              <button
                onClick={() => onRemove(toast.id)}
                className="text-text-muted hover:text-text-primary mt-0.5 flex-shrink-0"
              >
                <X size={14} />
              </button>
            </motion.div>
          );
        })}
      </AnimatePresence>
    </div>
  );
}
