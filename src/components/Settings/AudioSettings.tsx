import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppConfig } from "../../hooks/useSettings";
import { RefreshCw, Check } from "lucide-react";
import { EnergyMeter } from "../ui/EnergyMeter";

interface AudioDeviceInfo {
  name: string;
  channels: number;
  sample_rate: number;
  kind: string; // "mic" or "system"
}

interface AudioSettingsProps {
  config: AppConfig;
  onSave: (config: AppConfig) => Promise<void>;
  isCapturing: boolean;
}

export function AudioSettings({ config, onSave, isCapturing }: AudioSettingsProps) {
  const [devices, setDevices] = useState<AudioDeviceInfo[]>([]);
  const [source, setSource] = useState(config.audio.source);
  const [device, setDevice] = useState(config.audio.device);
  const [vadThreshold, setVadThreshold] = useState(config.audio.vad_threshold);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadDevices();
  }, []);

  const loadDevices = async () => {
    try {
      const devs = await invoke<AudioDeviceInfo[]>("list_audio_devices");
      setDevices(devs);
      setError(null);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to load audio devices";
      setError(msg);
      console.error("Failed to load devices:", err);
    }
  };

  const filteredDevices = devices;

  const handleSave = async () => {
    try {
      await onSave({
        ...config,
        audio: { ...config.audio, source, device, vad_threshold: vadThreshold },
      });
      setSaved(true);
      setError(null);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to save audio settings";
      setError(msg);
    }
  };

  return (
    <div className="flex flex-col">
      <div className="section">
        <div className="section-title">Input Source</div>

        {error && (
          <div className="mb-4 px-3 py-2 bg-danger-subtle border border-danger/20 rounded-lg text-[12px] text-danger">
            {error}
          </div>
        )}

        <div className="space-y-4">
          <div>
            <label className="label">Source</label>
            <select value={source} onChange={(e) => setSource(e.target.value)} className="select">
              <option value="system">System Audio (what&apos;s playing)</option>
              <option value="microphone">Microphone</option>
            </select>
          </div>

          <div>
            <label className="label">Device</label>
            <div className="flex gap-2">
              <select
                value={device}
                onChange={(e) => setDevice(e.target.value)}
                className="select flex-1"
              >
                <option value="default">Default</option>
                {filteredDevices.map((d) => (
                  <option key={d.name} value={d.name}>
                    {d.name} [{d.kind === "system" ? "system audio" : "mic"}]
                  </option>
                ))}
              </select>
              <button
                onClick={loadDevices}
                className="btn btn-ghost btn-sm"
                title="Refresh devices"
              >
                <RefreshCw size={14} />
              </button>
            </div>
            <p className="text-[11px] text-text-muted mt-1">
              {source === "system"
                ? "Select the monitor of your audio output (e.g., sink_default, Monitor of ...)"
                : "Select your microphone input device"}
            </p>
          </div>

          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="label mb-0">VAD Threshold</label>
              <span className="text-[13px] font-mono text-text-primary">
                {vadThreshold.toFixed(3)}
              </span>
            </div>
            <input
              type="range"
              min="0.001"
              max="0.1"
              step="0.001"
              value={vadThreshold}
              onChange={(e) => setVadThreshold(parseFloat(e.target.value))}
            />
            <div className="flex justify-between text-[11px] text-text-muted mt-1">
              <span>Sensitive (0.001)</span>
              <span>Default (0.01)</span>
              <span>Less Sensitive (0.1)</span>
            </div>
            <p className="text-[11px] text-text-muted mt-2">
              Adjust voice detection sensitivity. Lower values detect quieter speech.
            </p>
          </div>

          <EnergyMeter isCapturing={isCapturing} />
        </div>
      </div>

      <div className="section">
        <div className="section-title">Information</div>
        <div className="space-y-2 text-[13px] text-text-secondary">
          <div className="flex items-center gap-2">
            <span className="text-text-muted w-24">Sample rate</span>
            <span className="font-mono text-[12px]">{config.audio.sample_rate} Hz</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-text-muted w-24">Backend</span>
            <span>CPAL + PipeWire</span>
          </div>
        </div>
      </div>

      <div className="section flex justify-end">
        <button onClick={handleSave} className="btn btn-primary btn-sm gap-2">
          {saved ? <Check size={14} /> : null}
          {saved ? "Saved" : "Save"}
        </button>
      </div>
    </div>
  );
}
