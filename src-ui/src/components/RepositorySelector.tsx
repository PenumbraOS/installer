import React, { useState, useEffect } from "react";
import { useTauri, RepositoryInfo } from "../hooks/useTauri";

interface RepositorySelectorProps {
  deviceConnected: boolean;
  installing: boolean;
  onInstall: (selectedRepos: string[]) => void;
  onCancel: () => void;
}

export const RepositorySelector: React.FC<RepositorySelectorProps> = ({
  deviceConnected,
  installing,
  onInstall,
  onCancel,
}) => {
  const [repositories, setRepositories] = useState<RepositoryInfo[]>([]);
  const [selectedRepos, setSelectedRepos] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const api = useTauri();

  const loadRepositories = async () => {
    setLoading(true);
    setError(null);
    try {
      const repos = await api.getAvailableRepositories();
      setRepositories(repos);
    } catch (err) {
      console.error("Failed to load repositories:", err);
      setError(
        err instanceof Error ? err.message : "Failed to load repositories"
      );
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadRepositories();
  }, []);

  const handleRepoToggle = (repoName: string) => {
    setSelectedRepos((prev) =>
      prev.includes(repoName)
        ? prev.filter((r) => r !== repoName)
        : [...prev, repoName]
    );
  };

  const handleSelectAll = () => {
    if (selectedRepos.length === repositories.length) {
      setSelectedRepos([]);
    } else {
      setSelectedRepos(repositories.map((r) => r.name));
    }
  };

  const handleInstallSelected = () => {
    onInstall(selectedRepos);
  };

  const handleInstallAll = () => {
    onInstall([]);
  };

  const canInstall = deviceConnected && !installing;

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
      <h3 style={{ margin: "0 0 12px 0", fontSize: "18px", fontWeight: "600" }}>
        Available Repositories
      </h3>

      {error && (
        <div
          style={{
            padding: "8px",
            backgroundColor: "#ffebee",
            border: "1px solid #ffcdd2",
            borderRadius: "4px",
            fontSize: "14px",
            color: "#c62828",
            marginBottom: "12px",
          }}
        >
          {error}
        </div>
      )}

      {loading ? (
        <div style={{ padding: "12px", textAlign: "center", color: "#666" }}>
          Loading repositories...
        </div>
      ) : (
        <>
          <div
            style={{
              maxHeight: "150px",
              overflowY: "auto",
              marginBottom: "12px",
              border: "1px solid #e0e0e0",
              borderRadius: "4px",
              padding: "8px",
            }}
          >
            {repositories.length === 0 ? (
              <div
                style={{ padding: "12px", textAlign: "center", color: "#666" }}
              >
                No repositories available
              </div>
            ) : (
              <div
                style={{ display: "flex", flexDirection: "column", gap: "8px" }}
              >
                {repositories.map((repo) => (
                  <label
                    key={repo.name}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: "8px",
                      padding: "4px",
                      cursor: canInstall ? "pointer" : "not-allowed",
                      opacity: canInstall ? 1 : 0.6,
                    }}
                  >
                    <input
                      type="checkbox"
                      checked={selectedRepos.includes(repo.name)}
                      onChange={() => canInstall && handleRepoToggle(repo.name)}
                      disabled={!canInstall}
                      style={{
                        width: "16px",
                        height: "16px",
                        cursor: canInstall ? "pointer" : "not-allowed",
                      }}
                    />
                    <div style={{ flex: 1 }}>
                      <div style={{ fontSize: "14px", fontWeight: "500" }}>
                        {repo.name}
                      </div>
                      <div style={{ fontSize: "12px", color: "#666" }}>
                        {repo.owner}/{repo.repo}
                      </div>
                    </div>
                  </label>
                ))}
              </div>
            )}
          </div>

          {repositories.length > 0 && (
            <>
              <div style={{ marginBottom: "12px" }}>
                <button
                  onClick={handleSelectAll}
                  disabled={!canInstall}
                  style={{
                    padding: "4px 8px",
                    backgroundColor: "transparent",
                    color: canInstall ? "#2196F3" : "#ccc",
                    border: `1px solid ${canInstall ? "#2196F3" : "#ccc"}`,
                    borderRadius: "4px",
                    cursor: canInstall ? "pointer" : "not-allowed",
                    fontSize: "12px",
                  }}
                >
                  {selectedRepos.length === repositories.length
                    ? "Deselect All"
                    : "Select All"}
                </button>
              </div>

              <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
                <button
                  onClick={handleInstallSelected}
                  disabled={!canInstall || selectedRepos.length === 0}
                  style={{
                    padding: "8px 16px",
                    backgroundColor:
                      canInstall && selectedRepos.length > 0
                        ? "#4CAF50"
                        : "#ccc",
                    color: "white",
                    border: "none",
                    borderRadius: "4px",
                    cursor:
                      canInstall && selectedRepos.length > 0
                        ? "pointer"
                        : "not-allowed",
                    fontWeight: "500",
                  }}
                >
                  Install Selected ({selectedRepos.length})
                </button>

                <button
                  onClick={handleInstallAll}
                  disabled={!canInstall}
                  style={{
                    padding: "8px 16px",
                    backgroundColor: canInstall ? "#2196F3" : "#ccc",
                    color: "white",
                    border: "none",
                    borderRadius: "4px",
                    cursor: canInstall ? "pointer" : "not-allowed",
                    fontWeight: "500",
                  }}
                >
                  Install All
                </button>

                {installing && (
                  <button
                    onClick={onCancel}
                    style={{
                      padding: "8px 16px",
                      backgroundColor: "#f44336",
                      color: "white",
                      border: "none",
                      borderRadius: "4px",
                      cursor: "pointer",
                      fontWeight: "500",
                    }}
                  >
                    Cancel
                  </button>
                )}
              </div>
            </>
          )}

          {!deviceConnected && (
            <div
              style={{
                padding: "8px",
                backgroundColor: "#fff3e0",
                border: "1px solid #ffcc02",
                borderRadius: "4px",
                fontSize: "14px",
                color: "#f57c00",
                marginTop: "8px",
              }}
            >
              Connect a device to enable installation
            </div>
          )}
        </>
      )}
    </div>
  );
};
