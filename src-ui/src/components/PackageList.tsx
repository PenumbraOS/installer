import React, { useState, useEffect } from "react";
import { useTauri, PackageInfo } from "../hooks/useTauri";

interface PackageListProps {
  deviceConnected: boolean;
}

export const PackageList: React.FC<PackageListProps> = ({
  deviceConnected,
}) => {
  const [packages, setPackages] = useState<PackageInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const api = useTauri();

  const loadPackages = async () => {
    if (!deviceConnected) {
      setPackages([]);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const packageList = await api.listInstalledPackages();
      setPackages(packageList);
    } catch (err) {
      console.error("Failed to load packages:", err);
      setError(err instanceof Error ? err.message : "Failed to load packages");
      setPackages([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadPackages();
  }, [deviceConnected]);

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
          marginBottom: "12px",
        }}
      >
        <h3 style={{ margin: 0, fontSize: "18px", fontWeight: "600" }}>
          Installed Packages
        </h3>
        <button
          onClick={loadPackages}
          disabled={loading || !deviceConnected}
          style={{
            padding: "6px 12px",
            backgroundColor: deviceConnected ? "#2196F3" : "#ccc",
            color: "white",
            border: "none",
            borderRadius: "4px",
            cursor: loading || !deviceConnected ? "not-allowed" : "pointer",
            opacity: loading || !deviceConnected ? 0.6 : 1,
          }}
        >
          {loading ? "Loading..." : "Refresh"}
        </button>
      </div>

      {!deviceConnected && (
        <div
          style={{
            padding: "12px",
            backgroundColor: "#f5f5f5",
            borderRadius: "4px",
            fontSize: "14px",
            color: "#666",
            fontStyle: "italic",
          }}
        >
          Connect a device to view installed packages
        </div>
      )}

      {error && (
        <div
          style={{
            padding: "8px",
            backgroundColor: "#ffebee",
            border: "1px solid #ffcdd2",
            borderRadius: "4px",
            fontSize: "14px",
            color: "#c62828",
            marginBottom: "8px",
          }}
        >
          {error}
        </div>
      )}

      {deviceConnected && !loading && !error && (
        <div style={{ maxHeight: "200px", overflowY: "auto" }}>
          {packages.length === 0 ? (
            <div
              style={{
                padding: "12px",
                backgroundColor: "#f5f5f5",
                borderRadius: "4px",
                fontSize: "14px",
                color: "#666",
                textAlign: "center",
              }}
            >
              No relevant packages found
            </div>
          ) : (
            <div
              style={{ display: "flex", flexDirection: "column", gap: "8px" }}
            >
              {packages.map((pkg, index) => (
                <div
                  key={index}
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    padding: "8px",
                    backgroundColor: "#f9f9f9",
                    borderRadius: "4px",
                    border: "1px solid #e0e0e0",
                  }}
                >
                  <span
                    style={{
                      fontSize: "14px",
                      fontFamily: "monospace",
                      fontWeight: "500",
                    }}
                  >
                    {pkg.package_name}
                  </span>
                  {pkg.version && (
                    <span
                      style={{
                        fontSize: "12px",
                        color: "#666",
                        backgroundColor: "#e0e0e0",
                        padding: "2px 6px",
                        borderRadius: "3px",
                      }}
                    >
                      {pkg.version}
                    </span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
