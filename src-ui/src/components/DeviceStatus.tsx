import React, { useState, useEffect } from "react";
import { useTauri, DeviceInfo } from "../hooks/useTauri";

interface DeviceStatusProps {
  onDeviceChange?: (connected: boolean) => void;
}

export const DeviceStatus: React.FC<DeviceStatusProps> = ({
  onDeviceChange,
}) => {
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const api = useTauri();

  const checkDevice = async () => {
    setLoading(true);
    try {
      const info = await api.checkDeviceConnection();
      setDeviceInfo(info);
      onDeviceChange?.(info.connected);
    } catch (error) {
      console.error("Failed to check device:", error);
      setDeviceInfo({
        connected: false,
        device_count: 0,
        error_message: "Failed to check device connection",
      });
      onDeviceChange?.(false);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    checkDevice();
  }, []);

  const getStatusColor = () => {
    if (!deviceInfo) return "#999";
    if (deviceInfo.connected) return "#4CAF50";
    return "#f44336";
  };

  const getStatusText = () => {
    if (loading) return "Checking...";
    if (!deviceInfo) return "Unknown";
    if (deviceInfo.connected)
      return "Connected - Device ready for installation";
    if (deviceInfo.device_count === 0) return "No device connected";
    if (deviceInfo.device_count > 1) return "Multiple devices connected";
    return "Device not ready";
  };

  return (
    <div
      style={{
        padding: "16px",
        backgroundColor: "white",
        borderRadius: "8px",
        marginBottom: "16px",
        border: "1px solid #e0e0e0",
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: "8px",
        }}
      >
        <h3 style={{ margin: 0, fontSize: "18px", fontWeight: "600" }}>
          Device Status
        </h3>
        <button
          onClick={checkDevice}
          disabled={loading}
          style={{
            padding: "6px 12px",
            backgroundColor: "#2196F3",
            color: "white",
            border: "none",
            borderRadius: "4px",
            cursor: loading ? "not-allowed" : "pointer",
            opacity: loading ? 0.6 : 1,
          }}
        >
          {loading ? "Checking..." : "Refresh"}
        </button>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <div
          style={{
            width: "12px",
            height: "12px",
            borderRadius: "50%",
            backgroundColor: getStatusColor(),
          }}
        />
        <span style={{ fontSize: "14px", fontWeight: "500" }}>
          {getStatusText()}
        </span>
      </div>

      {deviceInfo?.error_message && (
        <div
          style={{
            marginTop: "8px",
            padding: "8px",
            backgroundColor: "#ffebee",
            border: "1px solid #ffcdd2",
            borderRadius: "4px",
            fontSize: "14px",
            color: "#c62828",
          }}
        >
          {deviceInfo.error_message}
        </div>
      )}
    </div>
  );
};
