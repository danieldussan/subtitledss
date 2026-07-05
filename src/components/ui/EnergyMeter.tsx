import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface EnergyMeterProps {
  isCapturing: boolean;
}

export function EnergyMeter({ isCapturing }: EnergyMeterProps) {
  const [energy, setEnergy] = useState<number>(0);
  const animationRef = useRef<number | undefined>(undefined);
  const lastUpdateRef = useRef<number>(0);

  useEffect(() => {
    if (isCapturing) {
      const updateEnergy = async () => {
        const now = performance.now();
        if (now - lastUpdateRef.current >= 50) {
          try {
            const level = await invoke<number>("get_audio_level");
            setEnergy(Math.min(level * 3, 1.0));
          } catch {
            setEnergy(0);
          }
          lastUpdateRef.current = now;
        }
        animationRef.current = requestAnimationFrame(updateEnergy);
      };
      animationRef.current = requestAnimationFrame(updateEnergy);
    } else {
      setEnergy(0);
    }

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [isCapturing]);

  const getEnergyColor = (level: number) => {
    if (level < 0.01) return "bg-text-muted";
    if (level < 0.3) return "bg-success";
    if (level < 0.7) return "bg-warning";
    return "bg-danger";
  };

  return (
    <div className="space-y-2">
      <label className="label mb-0">Audio Level</label>
      <div className="relative h-4 bg-bg-base rounded-full overflow-hidden">
        <div
          className={`absolute left-0 top-0 h-full transition-all duration-75 ${getEnergyColor(energy)}`}
          style={{ width: `${Math.min(energy * 100, 100)}%` }}
        />
      </div>
      <div className="flex items-center justify-between text-[11px] text-text-muted">
        <span>
          {isCapturing
            ? energy < 0.01
              ? "No signal"
              : energy < 0.3
                ? "Low"
                : energy < 0.7
                  ? "Medium"
                  : "High"
            : "Idle"}
        </span>
        <span className="font-mono">{(energy * 100).toFixed(1)}%</span>
      </div>
      {!isCapturing && (
        <p className="text-[11px] text-text-muted">Start capture to see audio levels</p>
      )}
    </div>
  );
}
