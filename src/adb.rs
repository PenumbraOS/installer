use crate::{InstallerError, Result};
use adb_client::{ADBDeviceExt, ADBServer, ADBServerDevice};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;

pub struct AdbManager {
    device: ADBServerDevice,
}

impl AdbManager {
    pub async fn connect() -> Result<Self> {
        // TODO: Integrate USB when https://github.com/cocool97/adb_client/issues/108 is fixed
        // // Use autodetect to find and connect to USB device
        // let device = ADBUSBDevice::autodetect().map_err(|e| match e {
        //     adb_client::RustADBError::DeviceNotFound(msg) => {
        //         if msg.contains("two Android devices") {
        //             InstallerError::MultipleDevices
        //         } else {
        //             InstallerError::NoDevice
        //         }
        //     }
        //     _ => InstallerError::Adb(format!("Failed to connect to USB device: {}", e)),
        // })?;

        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);
        let mut server = ADBServer::new(addr);

        let devices = server
            .devices()
            .map_err(|e| InstallerError::Adb(format!("Failed to list devices: {}", e)))?;

        match devices.len() {
            0 => Err(InstallerError::NoDevice),
            1 => {
                let device_info = devices.into_iter().next().unwrap();

                match device_info.state {
                    adb_client::DeviceState::Device => {
                        // TODO: Check if device is Pin
                        let device =
                            server
                                .get_device_by_name(&device_info.identifier)
                                .map_err(|e| {
                                    InstallerError::Adb(format!("Failed to get device: {}", e))
                                })?;

                        Ok(Self { device })
                    }
                    adb_client::DeviceState::Unauthorized => Err(InstallerError::Adb(
                        "Device unauthorized. Please enable USB debugging".to_string(),
                    )),
                    _ => Err(InstallerError::Adb(format!(
                        "Device not ready: {:?}",
                        device_info.state
                    ))),
                }
            }
            _ => Err(InstallerError::MultipleDevices),
        }
    }

    pub async fn install_apk(&mut self, path: &Path) -> Result<()> {
        self.device
            .install(path)
            .map_err(|e| InstallerError::ApkInstallation {
                apk: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                reason: format!("{}", e),
            })?;

        Ok(())
    }

    pub async fn uninstall_package(&mut self, package: &str) -> Result<()> {
        let _ = self
            .shell(&format!("pm uninstall --user 0 {}", package))
            .await;
        let _ = self.shell(&format!("pm uninstall {}", package)).await;

        Ok(())
    }

    pub async fn shell(&mut self, command: &str) -> Result<String> {
        let mut output = Vec::new();
        let cmd_parts: Vec<&str> = command.split_whitespace().collect();

        self.device
            .shell_command(&cmd_parts, &mut output)
            .map_err(|e| InstallerError::Adb(format!("Failed to run shell command: {}", e)))?;

        let output_str = String::from_utf8_lossy(&output);
        Ok(output_str.trim().to_string())
    }

    pub async fn push_file(&mut self, local: &Path, remote: &str) -> Result<()> {
        let mut file = std::fs::File::open(local)
            .map_err(|e| InstallerError::Adb(format!("Failed to open file: {}", e)))?;

        self.device
            .push(&mut file, &remote)
            .map_err(|e| InstallerError::Adb(format!("Push failed: {}", e)))?;

        Ok(())
    }

    pub async fn grant_permission(&mut self, package: &str, permission: &str) -> Result<()> {
        self.shell(&format!("pm grant {} {}", package, permission))
            .await?;
        Ok(())
    }

    pub async fn set_app_op(&mut self, package: &str, operation: &str, mode: &str) -> Result<()> {
        self.shell(&format!("appops set {} {} {}", package, operation, mode))
            .await?;
        Ok(())
    }

    pub async fn set_launcher(&mut self, component: &str) -> Result<()> {
        self.shell(&format!("cmd package set-home-activity {}", component))
            .await?;
        Ok(())
    }

    pub async fn create_directory(&mut self, path: &str) -> Result<()> {
        self.shell(&format!("mkdir -p {}", path)).await?;
        Ok(())
    }

    pub async fn remove_directory(&mut self, path: &str) -> Result<()> {
        self.shell(&format!("rm -rf {}", path)).await?;
        Ok(())
    }

    pub async fn remove_file(&mut self, path: &str) -> Result<()> {
        self.shell(&format!("rm -f {}", path)).await?;
        Ok(())
    }

    pub async fn file_exists(&mut self, path: &str) -> Result<bool> {
        let result = self
            .shell(&format!("[ -f {} ] && echo 'exists'", path))
            .await?;
        Ok(result.contains("exists"))
    }

    pub async fn write_file(&mut self, path: &str, content: &str) -> Result<()> {
        let escaped_content = content.replace('\'', "'\"'\"'");
        self.shell(&format!("echo '{}' > {}", escaped_content, path))
            .await?;
        Ok(())
    }

    pub async fn list_packages(&mut self, pattern: &str) -> Result<Vec<String>> {
        let output = self
            .shell(&format!(
                "pm list packages | grep {} | sed 's/package://'",
                pattern
            ))
            .await?;

        let packages: Vec<String> = output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        Ok(packages)
    }
}
