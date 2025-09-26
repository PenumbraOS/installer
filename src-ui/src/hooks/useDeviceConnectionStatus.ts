import { useEffect, useState } from "react";
import { DeviceInfo, useTauri } from "./useTauri";

export const useDeviceConnectionStatus = () => {
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | undefined>(
    undefined
  );
  const [loading, setLoading] = useState(false);

  const api = useTauri();

  const checkDevice = async () => {
    setLoading(true);
    try {
      const info = await api.checkDeviceConnection();
      setDeviceInfo(info);
    } catch (error) {
      console.error("Failed to check device:", error);
      setDeviceInfo({
        connected: false,
        error_message: "Failed to check device connection",
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    const interval = setInterval(checkDevice, 5000);

    checkDevice();

    return () => {
      clearInterval(interval);
    };
  }, []);

  return [deviceInfo, loading, checkDevice] as const;
};
