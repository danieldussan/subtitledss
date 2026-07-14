import { EnergyMeter } from "../ui/EnergyMeter";

interface AudioMeterCardProps {
  isCapturing: boolean;
}

export function AudioMeterCard({ isCapturing }: AudioMeterCardProps) {
  return (
    <div className="card">
      <div className="flex items-center justify-between px-4 py-3 border-b border-border-subtle">
        <span className="text-[13px] font-semibold text-text-primary">Audio Level</span>
      </div>
      <div className="p-4">
        {isCapturing ? (
          <EnergyMeter isCapturing={isCapturing} />
        ) : (
          <div className="text-center py-4 text-text-muted">
            <div className="text-[12px]">Audio unavailable</div>
            <div className="text-[11px] mt-1">Start capturing to see audio levels</div>
          </div>
        )}
      </div>
    </div>
  );
}
