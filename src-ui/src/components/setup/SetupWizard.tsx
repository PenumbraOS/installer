import React, {
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useRef,
} from "react";
import {
  Stepper,
  Text,
  Group,
  Button,
  Alert,
  Loader,
  Stack,
  Title,
  Modal,
} from "@mantine/core";
import { IconSettings } from "@tabler/icons-react";
import { PackageList } from "../PackageList";
import { RepositorySelector } from "../RepositorySelector";
import { ConsoleOutput } from "../ConsoleOutput";
import { NoDevice } from "../NoDevice";
import { useDeviceConnectionStatus } from "../../hooks/useDeviceConnectionStatus";
import { useSetupConfig } from "../../hooks/useSetupConfig";
import { AdbSource, SetupConfig, useTauri } from "../../hooks/useTauri";
import { AdbSection } from "./AdbSection";
import { GithubSection } from "./GithubSection";

const isRemoteSource = (
  source?: AdbSource | null
): source is Extract<AdbSource, { type: "remote_server" }> =>
  source?.type === "remote_server";

const githubPreview = (token?: string | null) => {
  if (!token) {
    return null;
  }

  if (token.length <= 4) {
    return `•••${token}`;
  }

  return `•••${token.slice(-4)}`;
};

type PendingFileState = {
  file: File;
  path?: string | null;
} | null;

interface WizardState {
  installing: boolean;
  activeStep: number;
  wizardComplete: boolean;
  hasCompletedWizard: boolean;
  settingsOpened: boolean;
  initialConfigEvaluated: boolean;
  adbMode: "file" | "remote";
  remoteUrl: string;
  remoteUrlDirty: boolean;
  githubTokenInput: string;
  pendingFile: PendingFileState;
  adbError: string | null;
  githubError: string | null;
  savingAdb: boolean;
  savingGithub: boolean;
}

type WizardAction =
  | { type: "restoreFromConfig"; config: SetupConfig | null }
  | { type: "markInitialEvaluated"; adbConfigured: boolean }
  | { type: "advanceToStep"; step: number }
  | { type: "setActiveStep"; step: number }
  | { type: "completeWizard" }
  | { type: "restartWizard" }
  | { type: "cancelWizard" }
  | { type: "setInstalling"; installing: boolean }
  | { type: "selectAdbFile"; pendingFile: PendingFileState }
  | { type: "resetPendingFile" }
  | { type: "updateRemoteUrl"; value: string; remoteMatchesStored: boolean }
  | { type: "setRemoteDirty"; dirty: boolean }
  | { type: "setAdbMode"; mode: "file" | "remote" }
  | { type: "setAdbSaving"; saving: boolean }
  | { type: "setGithubSaving"; saving: boolean }
  | { type: "setAdbError"; error: string | null }
  | { type: "setGithubError"; error: string | null }
  | { type: "setGithubInput"; value: string }
  | { type: "setSettingsOpen"; open: boolean }
  | { type: "remoteUrlSaved"; url: string }
  | { type: "clearRemoteSource" };

const initialWizardState: WizardState = {
  installing: false,
  activeStep: 0,
  wizardComplete: false,
  hasCompletedWizard: false,
  settingsOpened: false,
  initialConfigEvaluated: false,
  adbMode: "file",
  remoteUrl: "",
  remoteUrlDirty: false,
  githubTokenInput: "",
  pendingFile: null,
  adbError: null,
  githubError: null,
  savingAdb: false,
  savingGithub: false,
};

const wizardReducer = (
  state: WizardState,
  action: WizardAction
): WizardState => {
  switch (action.type) {
    case "restoreFromConfig": {
      const source = action.config?.adb_source;
      const isRemote = isRemoteSource(source);
      return {
        ...state,
        adbMode: isRemote ? "remote" : "file",
        remoteUrl: isRemote ? source.url : "",
        remoteUrlDirty: false,
        pendingFile: null,
        adbError: null,
        githubError: null,
        githubTokenInput: action.config?.github_token ?? "",
      };
    }
    case "markInitialEvaluated":
      if (state.initialConfigEvaluated) {
        return state;
      }
      if (action.adbConfigured) {
        return {
          ...state,
          initialConfigEvaluated: true,
          wizardComplete: true,
          hasCompletedWizard: true,
          activeStep: Math.max(state.activeStep, 1),
        };
      }
      return {
        ...state,
        initialConfigEvaluated: true,
      };
    case "advanceToStep":
      return {
        ...state,
        activeStep: Math.max(state.activeStep, action.step),
      };
    case "setActiveStep":
      return {
        ...state,
        activeStep: action.step,
      };
    case "completeWizard":
      return {
        ...state,
        wizardComplete: true,
        hasCompletedWizard: true,
        activeStep: Math.max(state.activeStep, 1),
      };
    case "restartWizard":
      return {
        ...state,
        wizardComplete: false,
        settingsOpened: false,
        activeStep: 0,
      };
    case "cancelWizard":
      return {
        ...state,
        wizardComplete: true,
        hasCompletedWizard: true,
        activeStep: 1,
      };
    case "setInstalling":
      return { ...state, installing: action.installing };
    case "selectAdbFile":
      return { ...state, pendingFile: action.pendingFile, adbError: null };
    case "resetPendingFile":
      return { ...state, pendingFile: null };
    case "updateRemoteUrl":
      return {
        ...state,
        remoteUrl: action.value,
        remoteUrlDirty: !action.remoteMatchesStored,
      };
    case "setRemoteDirty":
      return { ...state, remoteUrlDirty: action.dirty };
    case "setAdbMode":
      return { ...state, adbMode: action.mode };
    case "setAdbSaving":
      return { ...state, savingAdb: action.saving };
    case "setGithubSaving":
      return { ...state, savingGithub: action.saving };
    case "setAdbError":
      return { ...state, adbError: action.error };
    case "setGithubError":
      return { ...state, githubError: action.error };
    case "setGithubInput":
      return { ...state, githubTokenInput: action.value };
    case "setSettingsOpen":
      return { ...state, settingsOpened: action.open };
    case "remoteUrlSaved":
      return {
        ...state,
        adbMode: "remote",
        remoteUrl: action.url,
        remoteUrlDirty: false,
      };
    case "clearRemoteSource":
      return {
        ...state,
        adbMode: "file",
        remoteUrl: "",
        remoteUrlDirty: false,
      };
    default:
      return state;
  }
};

export const SetupWizard: React.FC = () => {
  const api = useTauri();
  const {
    config,
    loading: setupLoading,
    setAdbFromFile,
    setAdbFromBytes,
    setAdbRemote,
    clearAdb,
    setGithubToken,
  } = useSetupConfig();
  const [deviceInfo, deviceLoading, checkDevice] = useDeviceConnectionStatus();

  const [
    {
      installing,
      activeStep,
      wizardComplete,
      hasCompletedWizard,
      settingsOpened,
      initialConfigEvaluated,
      adbMode,
      remoteUrl,
      remoteUrlDirty,
      githubTokenInput,
      pendingFile,
      adbError,
      githubError,
      savingAdb,
      savingGithub,
    },
    dispatch,
  ] = useReducer(wizardReducer, initialWizardState);
  const fileInputResetRef = useRef<(() => void) | null>(null);

  const applyConfigToState = useCallback(
    (nextConfig: SetupConfig | null) => {
      dispatch({ type: "restoreFromConfig", config: nextConfig });
    },
    [dispatch]
  );

  const lastAppliedConfigSignature = useRef<string | null>(null);

  const getConfigSignature = useCallback((cfg: SetupConfig | null | undefined) => {
    if (!cfg) {
      return "null";
    }

    const source = cfg.adb_source;
    if (!source) {
      return JSON.stringify({ github_token: cfg.github_token ?? null });
    }

    return JSON.stringify({ github_token: cfg.github_token ?? null, adb_source: source });
  }, []);

  const restoreInputsFromConfig = useCallback(() => {
    applyConfigToState(config ?? null);
    lastAppliedConfigSignature.current = getConfigSignature(config);
  }, [applyConfigToState, config, getConfigSignature]);

  useEffect(() => {
    if (setupLoading) {
      return;
    }

    const signature = getConfigSignature(config);

    if (lastAppliedConfigSignature.current === signature) {
      return;
    }

    lastAppliedConfigSignature.current = signature;
    applyConfigToState(config ?? null);
  }, [applyConfigToState, config, getConfigSignature, setupLoading]);

  useEffect(() => {
    if (!setupLoading && !initialConfigEvaluated) {
      dispatch({
        type: "markInitialEvaluated",
        adbConfigured: Boolean(config?.adb_source),
      });
    }
  }, [config?.adb_source, initialConfigEvaluated, setupLoading]);

  const highestUnlockedStep = useMemo(() => {
    if (!config?.adb_source) {
      return 0;
    }

    return 1;
  }, [config?.adb_source]);

  const pendingFileName = useMemo(() => {
    if (!pendingFile) {
      return null;
    }

    return pendingFile.file.name || pendingFile.path || null;
  }, [pendingFile]);

  const pendingFileSource = useMemo(() => {
    if (!pendingFile) {
      return null;
    }

    return pendingFile.path || pendingFile.file.name;
  }, [pendingFile]);

  const remoteUrlNeedsSave = useMemo(() => {
    if (adbMode !== "remote") {
      return false;
    }

    if (!remoteUrlDirty) {
      return false;
    }

    return remoteUrl.trim().length > 0;
  }, [adbMode, remoteUrl, remoteUrlDirty]);

  const adbSummary = useMemo(() => {
    const source = config?.adb_source;
    if (source?.type === "local_copy") {
      return `Local file: ${source.original_filename}`;
    }

    if (source?.type === "remote_server") {
      return `Remote server: ${source.url}`;
    }

    return "Not configured";
  }, [config?.adb_source]);

  const handleStepClick = (step: number) => {
    if (step > highestUnlockedStep) {
      return;
    }

    dispatch({ type: "setActiveStep", step });
  };

  const handleInstall = async (selectedRepos: string[]) => {
    dispatch({ type: "setInstalling", installing: true });
    try {
      await api.installRepositories(selectedRepos);
    } catch (error) {
      console.error("Installation failed:", error);
    } finally {
      dispatch({ type: "setInstalling", installing: false });
    }
  };

  const handleCancel = async () => {
    try {
      await api.cancelInstallation();
    } catch (error) {
      console.error("Failed to cancel installation:", error);
    } finally {
      dispatch({ type: "setInstalling", installing: false });
    }
  };

  const handleFileSelection = useCallback(
    (file: File, inferredPath?: string | null) => {
      dispatch({
        type: "selectAdbFile",
        pendingFile: { file, path: inferredPath ?? null },
      });
    },
    [dispatch]
  );

  const handleFileFromPicker = useCallback(
    (file: File | null) => {
      if (!file) {
        return;
      }

      const withPath = file as unknown as { path?: string };
      handleFileSelection(file, withPath.path ?? null);
      fileInputResetRef.current?.();
    },
    [handleFileSelection]
  );

  const resetAdbPendingState = useCallback(() => {
    dispatch({ type: "resetPendingFile" });
    fileInputResetRef.current?.();
  }, [dispatch]);

  const handleRemoteUrlChange = useCallback(
    (value: string) => {
      const remoteSource = config?.adb_source;
      const matchesStoredRemote =
        isRemoteSource(remoteSource) && value === remoteSource.url;
      dispatch({
        type: "updateRemoteUrl",
        value,
        remoteMatchesStored: matchesStoredRemote,
      });
    },
    [config, dispatch]
  );

  const handleAdbModeChange = useCallback(
    (mode: "file" | "remote") => {
      dispatch({ type: "setAdbMode", mode });
    },
    [dispatch]
  );

  const handleGithubTokenChange = useCallback(
    (value: string) => {
      dispatch({ type: "setGithubInput", value });
    },
    [dispatch]
  );

  const openSettings = useCallback(() => {
    dispatch({ type: "setSettingsOpen", open: true });
  }, [dispatch]);

  const closeSettings = useCallback(() => {
    dispatch({ type: "setSettingsOpen", open: false });
  }, [dispatch]);

  const persistAdbChoice = useCallback(async () => {
    if (savingAdb) {
      return false;
    }

    dispatch({ type: "setAdbError", error: null });

    const currentSource = config?.adb_source;

    if (adbMode === "file") {
      if (pendingFile) {
        dispatch({ type: "setAdbSaving", saving: true });
        dispatch({ type: "setAdbMode", mode: "file" });
        try {
          if (pendingFile.path) {
            await setAdbFromFile(pendingFile.path);
          } else {
            const buffer = await pendingFile.file.arrayBuffer();
            const bytes = Array.from(new Uint8Array(buffer));
            const filenameCandidate =
              (pendingFile.file.name && pendingFile.file.name.trim().length > 0
                ? pendingFile.file.name
                : pendingFile.path) || "adb_key";
            const normalizedFilename =
              filenameCandidate.trim().length > 0
                ? filenameCandidate.trim()
                : "adb_key";
            await setAdbFromBytes(normalizedFilename, bytes);
          }
          resetAdbPendingState();
          return true;
        } catch (error) {
          const message =
            error instanceof Error ? error.message : String(error);
          dispatch({ type: "setAdbError", error: message });
          return false;
        } finally {
          dispatch({ type: "setAdbSaving", saving: false });
        }
      }

      if (currentSource?.type === "local_copy") {
        return true;
      }

      dispatch({
        type: "setAdbError",
        error: "Select an ADB key file before continuing.",
      });
      return false;
    }

    const trimmed = remoteUrl.trim();

    if (!trimmed) {
      if (isRemoteSource(currentSource)) {
        dispatch({ type: "setAdbSaving", saving: true });
        try {
          await clearAdb();
          dispatch({ type: "clearRemoteSource" });
          dispatch({
            type: "setAdbError",
            error:
              "Signing server cleared. Select a key file or enter a new URL before continuing.",
          });
          resetAdbPendingState();
        } catch (error) {
          const message =
            error instanceof Error ? error.message : String(error);
          dispatch({ type: "setAdbError", error: message });
        } finally {
          dispatch({ type: "setAdbSaving", saving: false });
        }
      } else {
        dispatch({
          type: "setAdbError",
          error: "Enter the signing server URL before continuing.",
        });
      }

      return false;
    }

    if (isRemoteSource(currentSource) && currentSource.url === trimmed) {
      dispatch({ type: "setRemoteDirty", dirty: false });
      return true;
    }

    dispatch({ type: "setAdbSaving", saving: true });
    try {
      await setAdbRemote(trimmed);
      dispatch({ type: "remoteUrlSaved", url: trimmed });
      resetAdbPendingState();
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      dispatch({ type: "setAdbError", error: message });
      return false;
    } finally {
      dispatch({ type: "setAdbSaving", saving: false });
    }
  }, [
    adbMode,
    clearAdb,
    config?.adb_source,
    dispatch,
    pendingFile,
    remoteUrl,
    resetAdbPendingState,
    savingAdb,
    setAdbFromBytes,
    setAdbFromFile,
    setAdbRemote,
  ]);

  const persistGithubToken = useCallback(async () => {
    if (savingGithub) {
      return false;
    }

    dispatch({ type: "setGithubError", error: null });

    const trimmed = githubTokenInput.trim();
    const normalized = trimmed.length > 0 ? trimmed : null;
    const current = config?.github_token ?? null;

    if (normalized === current) {
      return true;
    }

    dispatch({ type: "setGithubSaving", saving: true });
    try {
      await setGithubToken(normalized);
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      dispatch({ type: "setGithubError", error: message });
      return false;
    } finally {
      dispatch({ type: "setGithubSaving", saving: false });
    }
  }, [
    config?.github_token,
    dispatch,
    githubTokenInput,
    savingGithub,
    setGithubToken,
  ]);

  const handleContinueFromAdb = useCallback(async () => {
    const saved = await persistAdbChoice();
    if (saved) {
      dispatch({ type: "advanceToStep", step: 1 });
    }
  }, [dispatch, persistAdbChoice]);

  const handleContinueFromGithub = useCallback(async () => {
    const saved = await persistGithubToken();
    if (saved) {
      dispatch({ type: "completeWizard" });
    }
  }, [dispatch, persistGithubToken]);

  const handleRestartWizard = useCallback(() => {
    restoreInputsFromConfig();
    dispatch({ type: "restartWizard" });
  }, [dispatch, restoreInputsFromConfig]);

  const handleCancelWizard = useCallback(() => {
    restoreInputsFromConfig();
    dispatch({ type: "cancelWizard" });
  }, [dispatch, restoreInputsFromConfig]);

  const handleSaveSettings = useCallback(async () => {
    const adbSaved = await persistAdbChoice();
    if (!adbSaved) {
      return;
    }

    const githubSaved = await persistGithubToken();
    if (!githubSaved) {
      return;
    }

    dispatch({ type: "setSettingsOpen", open: false });
  }, [dispatch, persistAdbChoice, persistGithubToken]);

  const wizardAdbFooter = (
    <Group justify="space-between">
      {hasCompletedWizard ? (
        <Button
          variant="subtle"
          color="gray"
          onClick={handleCancelWizard}
          disabled={savingAdb}
        >
          Cancel
        </Button>
      ) : (
        <div />
      )}
      <Button onClick={() => void handleContinueFromAdb()} loading={savingAdb}>
        Continue
      </Button>
    </Group>
  );

  const wizardGithubFooter = (
    <Group justify="space-between">
      <Group gap="sm">
        <Button
          variant="subtle"
          onClick={() => dispatch({ type: "setActiveStep", step: 0 })}
        >
          Back
        </Button>
        {hasCompletedWizard && (
          <Button
            variant="subtle"
            color="gray"
            onClick={handleCancelWizard}
            disabled={savingGithub}
          >
            Cancel
          </Button>
        )}
      </Group>
      <Button
        onClick={() => void handleContinueFromGithub()}
        loading={savingGithub}
      >
        Continue
      </Button>
    </Group>
  );

  if (!wizardComplete) {
    return (
      <Stepper
        active={activeStep}
        onStepClick={handleStepClick}
        allowNextStepsSelect={false}
      >
        <Stepper.Step
          label="ADB access"
          description="Provide a key or signing server"
        >
          {setupLoading && !config ? (
            <Group justify="center" mt="sm">
              <Loader size="sm" />
              <Text size="sm" c="dimmed">
                Loading saved setup…
              </Text>
            </Group>
          ) : (
            <AdbSection
              introText="We need a trusted ADB key to talk to the Ai Pin. Upload your key file or point to a signing server."
              footer={wizardAdbFooter}
              error={adbError}
              adbMode={adbMode}
              onModeChange={handleAdbModeChange}
              saving={savingAdb}
              pendingFileName={pendingFileName}
              pendingFileSource={pendingFileSource}
              adbSource={config?.adb_source ?? null}
              remoteUrl={remoteUrl}
              onRemoteUrlChange={handleRemoteUrlChange}
              remoteUrlNeedsSave={remoteUrlNeedsSave}
              onFilePicked={handleFileFromPicker}
              fileInputResetRef={fileInputResetRef}
            />
          )}
        </Stepper.Step>
        <Stepper.Step
          label="GitHub token"
          description="Optional rate-limit protection"
        >
          <GithubSection
            introText="Provide a GitHub personal access token to avoid API rate limits when downloading releases. Leave it blank to skip."
            footer={wizardGithubFooter}
            error={githubError}
            tokenValue={githubTokenInput}
            onTokenChange={handleGithubTokenChange}
            saving={savingGithub}
            storedTokenPreview={githubPreview(config?.github_token)}
          />
        </Stepper.Step>
      </Stepper>
    );
  }

  return (
    <>
      <Modal
        opened={settingsOpened}
        onClose={closeSettings}
        title="Installer settings"
        size="lg"
      >
        <Stack gap="xl">
          <Stack gap="md">
            <Title order={4}>ADB access</Title>
            <AdbSection
              introText="Upload a new key file or update the signing server to change how the installer connects."
              error={adbError}
              adbMode={adbMode}
              onModeChange={handleAdbModeChange}
              saving={savingAdb}
              pendingFileName={pendingFileName}
              pendingFileSource={pendingFileSource}
              adbSource={config?.adb_source ?? null}
              remoteUrl={remoteUrl}
              onRemoteUrlChange={handleRemoteUrlChange}
              remoteUrlNeedsSave={remoteUrlNeedsSave}
              onFilePicked={handleFileFromPicker}
              fileInputResetRef={fileInputResetRef}
            />
          </Stack>
          <Stack gap="md">
            <Title order={4}>GitHub token</Title>
            <GithubSection
              introText="Store an optional GitHub personal access token to raise rate limits when fetching releases."
              error={githubError}
              tokenValue={githubTokenInput}
              onTokenChange={handleGithubTokenChange}
              saving={savingGithub}
              storedTokenPreview={githubPreview(config?.github_token)}
            />
          </Stack>
          <Group justify="space-between">
            <Button
              variant="subtle"
              color="gray"
              onClick={handleRestartWizard}
              disabled={savingAdb || savingGithub}
            >
              Re-run wizard
            </Button>
            <Group gap="sm">
              <Button
                variant="default"
                color="gray"
                onClick={() => {
                  restoreInputsFromConfig();
                  closeSettings();
                }}
                disabled={savingAdb || savingGithub}
              >
                Cancel
              </Button>
              <Button
                onClick={() => void handleSaveSettings()}
                loading={savingAdb || savingGithub}
              >
                Save changes
              </Button>
            </Group>
          </Group>
        </Stack>
      </Modal>

      <Stack gap="xl">
        <Group justify="space-between" align="flex-start">
          <Stack gap={4}>
            <Title order={3}>Installer ready</Title>
            <Text size="sm" c="dimmed">
              ADB access: {adbSummary}
            </Text>
            <Text size="sm" c="dimmed">
              GitHub token: {config?.github_token ? "Saved" : "Not provided"}
            </Text>
          </Stack>
          <Button
            variant="light"
            leftSection={<IconSettings size={16} />}
            onClick={openSettings}
          >
            Settings
          </Button>
        </Group>

        {deviceInfo?.connected ? (
          <Stack gap="lg">
            <Alert color="teal" variant="light">
              Device connected. You can review packages and start the
              installation.
            </Alert>
            <PackageList deviceConnected={true} />
            <RepositorySelector
              deviceConnected={true}
              installing={installing}
              onInstall={handleInstall}
              onCancel={handleCancel}
            />
            <ConsoleOutput installing={installing} />
          </Stack>
        ) : (
          <NoDevice
            onReload={checkDevice}
            loading={deviceLoading}
            message={deviceInfo?.error_message}
          />
        )}
      </Stack>
    </>
  );
};

export default SetupWizard;
