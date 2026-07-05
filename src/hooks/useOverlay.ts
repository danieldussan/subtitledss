import { useState, useCallback } from "react";

export function useOverlay() {
  const [isVisible, setIsVisible] = useState(true);
  const [text, setText] = useState("");
  const [position, setPosition] = useState({ x: 100, y: 500 });

  const show = useCallback(() => setIsVisible(true), []);
  const hide = useCallback(() => setIsVisible(false), []);
  const toggle = useCallback(() => setIsVisible((v) => !v), []);

  const updateText = useCallback((newText: string) => {
    setText(newText);
    if (newText) {
      setIsVisible(true);
    }
  }, []);

  const move = useCallback((x: number, y: number) => {
    setPosition({ x, y });
  }, []);

  return { isVisible, text, position, show, hide, toggle, updateText, move };
}
