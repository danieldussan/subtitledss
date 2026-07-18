import { useState } from "react";
import { Check, Zap, Loader2, AlertCircle } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { AppConfig } from "../../hooks/useSettings";

interface AiSettingsProps {
  config: AppConfig;
  onSave: (config: AppConfig) => Promise<void>;
}

interface AiProviderOption {
  value: string;
  label: string;
  defaultUrl: string;
  needsApiKey: boolean;
}

const PROVIDERS: AiProviderOption[] = [
  { value: "Ollama", label: "Ollama", defaultUrl: "http://localhost:11434/v1", needsApiKey: false },
  {
    value: "LmStudio",
    label: "LM Studio",
    defaultUrl: "http://localhost:1234/v1",
    needsApiKey: false,
  },
  {
    value: "DeepSeek",
    label: "DeepSeek",
    defaultUrl: "https://api.deepseek.com/v1",
    needsApiKey: true,
  },
];

export function AiSettings({ config, onSave }: AiSettingsProps) {
  const [provider, setProvider] = useState(config.ai.provider);
  const [baseUrl, setBaseUrl] = useState(config.ai.base_url);
  const [apiKey, setApiKey] = useState(config.ai.api_key || "");
  const [model, setModel] = useState(config.ai.model);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<"success" | "error" | null>(null);
  const [testMessage, setTestMessage] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const handleProviderChange = (newProvider: string) => {
    setProvider(newProvider);
    const p = PROVIDERS.find((p) => p.value === newProvider);
    if (p) {
      setBaseUrl(p.defaultUrl);
      setApiKey("");
    }
  };

  const handleSave = async () => {
    try {
      await onSave({
        ...config,
        ai: {
          provider,
          base_url: baseUrl,
          api_key: apiKey || null,
          model,
        },
      });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to save AI settings:", err);
    }
  };

  const handleTest = async () => {
    try {
      setTesting(true);
      setTestResult(null);
      setTestMessage(null);
      const result = await invoke<string>("test_ai_connection", {
        config: { provider, base_url: baseUrl, api_key: apiKey || null, model },
      });
      setTestResult("success");
      setTestMessage(result);
    } catch (err) {
      setTestResult("error");
      setTestMessage(String(err));
    } finally {
      setTesting(false);
    }
  };

  return (
    <div className="section">
      <div className="section-title">AI Provider</div>
      <p className="text-[12px] text-text-muted mb-4">
        Configure the AI provider for summaries and chat
      </p>

      <div className="space-y-4">
        {/* Provider Selection */}
        <div>
          <label className="label">Provider</label>
          <div className="space-y-2">
            {PROVIDERS.map((p) => (
              <button
                key={p.value}
                onClick={() => handleProviderChange(p.value)}
                className={`w-full flex items-center gap-3 p-3 rounded-lg text-left transition-all ${
                  provider === p.value
                    ? "bg-accent-subtle border border-accent/30"
                    : "bg-bg-base border border-border-subtle hover:border-border-default"
                }`}
              >
                <div
                  className={`w-4 h-4 rounded-full border-2 flex items-center justify-center flex-shrink-0 ${
                    provider === p.value ? "border-accent" : "border-border-strong"
                  }`}
                >
                  {provider === p.value && <div className="w-2 h-2 rounded-full bg-accent" />}
                </div>
                <div>
                  <div className="text-[13px] font-medium text-text-primary">{p.label}</div>
                  <div className="text-[11px] text-text-muted">{p.defaultUrl}</div>
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Base URL */}
        <div>
          <label className="label">Base URL</label>
          <input
            type="text"
            value={baseUrl}
            onChange={(e) => setBaseUrl(e.target.value)}
            className="input"
            placeholder="http://localhost:11434/v1"
          />
        </div>

        {/* API Key (only for DeepSeek) */}
        {PROVIDERS.find((p) => p.value === provider)?.needsApiKey && (
          <div>
            <label className="label">API Key</label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              className="input"
              placeholder="sk-..."
            />
          </div>
        )}

        {/* Model */}
        <div>
          <label className="label">Model</label>
          <input
            type="text"
            value={model}
            onChange={(e) => setModel(e.target.value)}
            className="input"
            placeholder="llama3.2"
          />
        </div>

        {/* Test & Save */}
        <div className="flex items-center gap-3">
          <button onClick={handleTest} disabled={testing} className="btn btn-ghost btn-sm gap-2">
            {testing ? (
              <Loader2 size={14} className="animate-spin" />
            ) : testResult === "success" ? (
              <Check size={14} className="text-success" />
            ) : testResult === "error" ? (
              <AlertCircle size={14} className="text-danger" />
            ) : (
              <Zap size={14} />
            )}
            {testing ? "Testing..." : testResult === "success" ? "Connected" : "Test Connection"}
          </button>

          <div className="flex-1" />

          <button onClick={handleSave} className="btn btn-primary btn-sm gap-2">
            {saved ? <Check size={14} /> : null}
            {saved ? "Saved" : "Save"}
          </button>
        </div>

        {testMessage && (
          <div
            className={`text-[12px] mt-2 ${testResult === "success" ? "text-success" : "text-danger"}`}
          >
            {testMessage}
          </div>
        )}
      </div>
    </div>
  );
}
