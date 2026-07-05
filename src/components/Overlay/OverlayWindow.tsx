import { motion, AnimatePresence } from "framer-motion";

interface OverlayWindowProps {
  text: string;
  isVisible: boolean;
  position: { x: number; y: number };
}

export function OverlayWindow({ text, isVisible, position }: OverlayWindowProps) {
  return (
    <AnimatePresence>
      {isVisible && text && (
        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 12 }}
          transition={{ duration: 0.25, ease: [0.25, 0.46, 0.45, 0.94] }}
          className="fixed z-50 pointer-events-none"
          style={{ left: position.x, top: position.y }}
        >
          <div
            className="px-5 py-2.5 rounded-xl backdrop-blur-md max-w-[80vw] shadow-lg"
            style={{
              background: "rgba(12, 15, 20, 0.85)",
              border: "1px solid rgba(91, 141, 239, 0.15)",
            }}
          >
            <p className="text-white text-xl font-medium text-center leading-relaxed tracking-wide">
              {text}
            </p>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
