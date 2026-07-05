import { motion } from "framer-motion";

interface SubtitleTextProps {
  text: string;
  fontSize?: number;
  color?: string;
  className?: string;
}

export function SubtitleText({
  text,
  fontSize = 24,
  color = "#ffffff",
  className = "",
}: SubtitleTextProps) {
  if (!text) return null;

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.2 }}
      className={`text-center tracking-wide ${className}`}
      style={{ fontSize: `${fontSize}px`, color }}
    >
      {text}
    </motion.div>
  );
}
