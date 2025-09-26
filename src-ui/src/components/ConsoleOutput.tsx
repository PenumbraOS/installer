import React, { useState, useEffect, useRef } from "react";
import { useInstallationProgress } from "../hooks/useTauri";

interface ConsoleOutputProps {
  installing: boolean;
}

export const ConsoleOutput: React.FC<ConsoleOutputProps> = ({ installing }) => {
  const [output, setOutput] = useState<string[]>([]);
  const outputRef = useRef<HTMLDivElement>(null);

  const addMessage = (message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setOutput((prev) => [...prev, `[${timestamp}] ${message}`]);
  };

  useInstallationProgress(addMessage);

  const clearOutput = () => {
    setOutput([]);
  };

  const exportLogs = () => {
    const logsText = output.join("\n");
    const blob = new Blob([logsText], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `penumbra-installer-logs-${Date.now()}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (outputRef.current) {
      outputRef.current.scrollTop = outputRef.current.scrollHeight;
    }
  }, [output]);

  // Add initial message when installation starts
  useEffect(() => {
    if (installing && output.length === 0) {
      addMessage("Ready to start installation...");
    }
  }, [installing]);

  return (
    <div
      style={{
        padding: "16px",
        backgroundColor: "white",
        borderRadius: "8px",
        border: "1px solid #e0e0e0",
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: "12px",
        }}
      >
        <h3 style={{ margin: 0, fontSize: "18px", fontWeight: "600" }}>
          Installation Progress
        </h3>
        <div style={{ display: "flex", gap: "8px" }}>
          <button
            onClick={clearOutput}
            disabled={output.length === 0}
            style={{
              padding: "4px 8px",
              backgroundColor: output.length > 0 ? "#f44336" : "#ccc",
              color: "white",
              border: "none",
              borderRadius: "4px",
              cursor: output.length > 0 ? "pointer" : "not-allowed",
              fontSize: "12px",
            }}
          >
            Clear
          </button>
          <button
            onClick={exportLogs}
            disabled={output.length === 0}
            style={{
              padding: "4px 8px",
              backgroundColor: output.length > 0 ? "#2196F3" : "#ccc",
              color: "white",
              border: "none",
              borderRadius: "4px",
              cursor: output.length > 0 ? "pointer" : "not-allowed",
              fontSize: "12px",
            }}
            title="Export logs"
          >
            ⤓
          </button>
        </div>
      </div>

      <div
        ref={outputRef}
        style={{
          height: "200px",
          overflowY: "auto",
          backgroundColor: "#1a1a1a",
          color: "#f0f0f0",
          padding: "12px",
          borderRadius: "4px",
          fontFamily: 'Monaco, Consolas, "Courier New", monospace',
          fontSize: "13px",
          lineHeight: "1.4",
          border: "1px solid #333",
        }}
      >
        {output.length === 0 ? (
          <div style={{ color: "#888", fontStyle: "italic" }}>
            Installation output will appear here...
          </div>
        ) : (
          output.map((line, index) => (
            <div
              key={index}
              style={{
                marginBottom: "2px",
                whiteSpace: "pre-wrap",
                wordBreak: "break-word",
              }}
            >
              {line}
            </div>
          ))
        )}
        {installing && (
          <div
            style={{
              color: "#4CAF50",
              fontWeight: "bold",
              marginTop: "8px",
            }}
          >
            ● Installation in progress...
          </div>
        )}
      </div>
    </div>
  );
};
