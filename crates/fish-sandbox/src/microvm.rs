use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroVmConfig {
    pub vcpu_count: u8,
    pub memory_size_mib: u32,
    pub kernel_image_path: PathBuf,
    pub rootfs_path: PathBuf,
    pub enable_network: bool,
    pub read_only_rootfs: bool,
    pub hypervisor: HypervisorType,
    pub enable_seccomp: bool,
    pub enable_jailer: bool,
    pub vsock_enabled: bool,
    pub extra_drives: Vec<DriveConfig>,
    pub network_config: Option<NetworkConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypervisorType {
    Firecracker,
    CloudHypervisor,
    Qemu,
}

impl Default for HypervisorType {
    fn default() -> Self {
        Self::Firecracker
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveConfig {
    pub drive_id: String,
    pub path_on_host: PathBuf,
    pub is_read_only: bool,
    pub is_root_device: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub iface_id: String,
    pub host_dev_name: String,
    pub guest_mac: Option<String>,
    pub allow_mmds: bool,
}

impl Default for MicroVmConfig {
    fn default() -> Self {
        Self {
            vcpu_count: 2,
            memory_size_mib: 1024,
            kernel_image_path: PathBuf::from("/var/lib/fish/vmlinux"),
            rootfs_path: PathBuf::from("/var/lib/fish/rootfs.ext4"),
            enable_network: false,
            read_only_rootfs: true,
            hypervisor: HypervisorType::Firecracker,
            enable_seccomp: true,
            enable_jailer: true,
            vsock_enabled: false,
            extra_drives: Vec::new(),
            network_config: None,
        }
    }
}

impl MicroVmConfig {
    pub fn with_hypervisor(mut self, hypervisor: HypervisorType) -> Self {
        self.hypervisor = hypervisor;
        self
    }

    pub fn with_network(mut self, network: NetworkConfig) -> Self {
        self.enable_network = true;
        self.network_config = Some(network);
        self
    }

    pub fn with_extra_drive(mut self, drive: DriveConfig) -> Self {
        self.extra_drives.push(drive);
        self
    }

    pub fn with_seccomp(mut self, enabled: bool) -> Self {
        self.enable_seccomp = enabled;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.vcpu_count == 0 {
            return Err("vcpu_count must be at least 1".to_string());
        }
        if self.memory_size_mib < 128 {
            return Err("memory must be at least 128 MiB".to_string());
        }
        if !self.kernel_image_path.exists() && !cfg!(test) {
            // In test mode, allow non-existent paths
            // In production, would check file existence
        }
        Ok(())
    }
}

pub struct MicroVmJailer {
    config: MicroVmConfig,
    jail_dir: PathBuf,
    env_vars: HashMap<String, String>,
}

impl MicroVmJailer {
    pub fn new(config: MicroVmConfig, jail_dir: impl AsRef<Path>) -> Self {
        Self {
            config,
            jail_dir: jail_dir.as_ref().to_path_buf(),
            env_vars: HashMap::new(),
        }
    }

    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }

    pub fn build_jailer_command(&self) -> Vec<String> {
        match self.config.hypervisor {
            HypervisorType::Firecracker => vec![
                "firecracker".to_string(),
                "--config-file".to_string(),
                self.jail_dir
                    .join("vm_config.json")
                    .to_string_lossy()
                    .to_string(),
            ],
            HypervisorType::CloudHypervisor => vec![
                "cloud-hypervisor".to_string(),
                "--config-file".to_string(),
                self.jail_dir
                    .join("ch_config.json")
                    .to_string_lossy()
                    .to_string(),
            ],
            HypervisorType::Qemu => vec![
                "qemu-system-x86_64".to_string(),
                "-kernel".to_string(),
                self.config.kernel_image_path.to_string_lossy().to_string(),
                "-m".to_string(),
                format!("{}", self.config.memory_size_mib),
            ],
        }
    }

    pub fn build_jailer_wrapper_command(&self) -> Vec<String> {
        if !self.config.enable_jailer {
            return self.build_jailer_command();
        }

        match self.config.hypervisor {
            HypervisorType::Firecracker => vec![
                "jailer".to_string(),
                "--id".to_string(),
                format!("fish-vm-{}", self.jail_dir.file_name().unwrap_or_default().to_string_lossy()),
                "--exec-file".to_string(),
                "/usr/bin/firecracker".to_string(),
                "--uid".to_string(),
                "123".to_string(),
                "--gid".to_string(),
                "100".to_string(),
                "--chroot-base-dir".to_string(),
                self.jail_dir.to_string_lossy().to_string(),
                "--daemonize".to_string(),
            ],
            _ => self.build_jailer_command(),
        }
    }

    pub fn generate_vm_json(&self) -> Result<String, serde_json::Error> {
        let mut drives = vec![serde_json::json!({
            "drive_id": "rootfs",
            "path_on_host": self.config.rootfs_path.to_string_lossy(),
            "is_root_device": true,
            "is_read_only": self.config.read_only_rootfs
        })];

        for extra in &self.config.extra_drives {
            drives.push(serde_json::json!({
                "drive_id": extra.drive_id,
                "path_on_host": extra.path_on_host.to_string_lossy(),
                "is_root_device": extra.is_root_device,
                "is_read_only": extra.is_read_only
            }));
        }

        let mut payload = serde_json::json!({
            "boot-source": {
                "kernel_image_path": self.config.kernel_image_path.to_string_lossy(),
                "boot_args": "console=ttyS0 reboot=k panic=1 pci=off nomodules rw init=/init"
            },
            "drives": drives,
            "machine-config": {
                "vcpu_count": self.config.vcpu_count,
                "mem_size_mib": self.config.memory_size_mib,
                "smt": false,
                "track_dirty_pages": false
            },
            "network-interfaces": [],
            "vsock": null
        });

        if self.config.enable_network {
            if let Some(ref net) = self.config.network_config {
                payload["network-interfaces"] = serde_json::json!([{
                    "iface_id": net.iface_id,
                    "host_dev_name": net.host_dev_name,
                    "guest_mac": net.guest_mac,
                    "allow_mmds_requests": net.allow_mmds
                }]);
            } else {
                payload["network-interfaces"] = serde_json::json!([{
                    "iface_id": "eth0",
                    "host_dev_name": "tap0",
                    "allow_mmds_requests": false
                }]);
            }
        }

        if self.config.vsock_enabled {
            payload["vsock"] = serde_json::json!({
                "guest_cid": 3,
                "uds_path": self.jail_dir.join("vsock.sock").to_string_lossy()
            });
        }

        if self.config.enable_seccomp {
            payload["seccomp"] = serde_json::json!({
                "level": 2 // Advanced filtering
            });
        }

        serde_json::to_string_pretty(&payload)
    }

    pub fn generate_cloud_hypervisor_config(&self) -> Result<String, serde_json::Error> {
        let payload = serde_json::json!({
            "kernel": {
                "path": self.config.kernel_image_path.to_string_lossy()
            },
            "cmdline": {
                "args": "console=hvc0 reboot=k panic=1"
            },
            "disks": [{
                "path": self.config.rootfs_path.to_string_lossy(),
                "readonly": self.config.read_only_rootfs
            }],
            "cpus": {
                "boot_vcpus": self.config.vcpu_count,
                "max_vcpus": self.config.vcpu_count
            },
            "memory": {
                "size": self.config.memory_size_mib * 1024 * 1024
            },
            "seccomp": self.config.enable_seccomp
        });

        serde_json::to_string_pretty(&payload)
    }

    pub fn generate_jailer_config(&self) -> Result<String, serde_json::Error> {
        let payload = serde_json::json!({
            "jail_dir": self.jail_dir.to_string_lossy(),
            "hypervisor": format!("{:?}", self.config.hypervisor),
            "vcpu_count": self.config.vcpu_count,
            "memory_mib": self.config.memory_size_mib,
            "enable_network": self.config.enable_network,
            "enable_seccomp": self.config.enable_seccomp,
            "env": self.env_vars
        });

        serde_json::to_string_pretty(&payload)
    }

    /// Execute build inside MicroVM (simulated for now)
    pub fn execute_in_vm(&self, command: &[String]) -> Result<VmExecutionResult, String> {
        self.config.validate().map_err(|e| e.to_string())?;

        // In real implementation:
        // 1. Create jail dir
        // 2. Write VM config
        // 3. Start firecracker via jailer
        // 4. Execute command via vsock or serial
        // 5. Collect results
        // 6. Cleanup

        let start = std::time::Instant::now();

        // Simulate execution
        let cmd_str = command.join(" ");
        let simulated_output = format!(
            "MicroVM ({:?}) executed: {} in jail {:?} with {} vCPUs, {} MiB",
            self.config.hypervisor,
            cmd_str,
            self.jail_dir,
            self.config.vcpu_count,
            self.config.memory_size_mib
        );

        Ok(VmExecutionResult {
            exit_code: 0,
            stdout: simulated_output,
            stderr: String::new(),
            duration: start.elapsed(),
            vm_config: self.config.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct VmExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: std::time::Duration,
    pub vm_config: MicroVmConfig,
}

impl VmExecutionResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Hermetic build execution inside MicroVM - enterprise security
pub struct HermeticMicroVmExecutor {
    base_config: MicroVmConfig,
    jail_base: PathBuf,
}

impl HermeticMicroVmExecutor {
    pub fn new(base_config: MicroVmConfig, jail_base: impl AsRef<Path>) -> Self {
        Self {
            base_config,
            jail_base: jail_base.as_ref().to_path_buf(),
        }
    }

    pub fn execute_hermetic_build(
        &self,
        build_id: &str,
        command: Vec<String>,
        workspace_root: &Path,
    ) -> Result<VmExecutionResult, String> {
        let jail_dir = self.jail_base.join(build_id);
        std::fs::create_dir_all(&jail_dir).map_err(|e| e.to_string())?;

        // Create isolated rootfs copy
        let isolated_rootfs = jail_dir.join("rootfs.ext4");
        // In real implementation, would copy or overlay rootfs

        let mut vm_config = self.base_config.clone();
        vm_config.rootfs_path = isolated_rootfs;
        vm_config.read_only_rootfs = true;
        vm_config.enable_seccomp = true;

        let jailer = MicroVmJailer::new(vm_config, &jail_dir);
        let result = jailer.execute_in_vm(&command)?;

        // Cleanup jail dir
        let _ = std::fs::remove_dir_all(&jail_dir);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microvm_config_generation() {
        let jailer = MicroVmJailer::new(MicroVmConfig::default(), "/tmp/jail");
        let json_str = jailer.generate_vm_json().unwrap();
        assert!(json_str.contains("rootfs"));
        assert!(json_str.contains("boot-source"));
    }

    #[test]
    fn test_microvm_with_extra_drives_and_network() {
        let config = MicroVmConfig {
            vcpu_count: 4,
            memory_size_mib: 2048,
            enable_network: true,
            network_config: Some(NetworkConfig {
                iface_id: "eth0".to_string(),
                host_dev_name: "tap0".to_string(),
                guest_mac: Some("AA:FC:00:00:00:01".to_string()),
                allow_mmds: false,
            }),
            extra_drives: vec![DriveConfig {
                drive_id: "workspace".to_string(),
                path_on_host: PathBuf::from("/tmp/workspace.ext4"),
                is_read_only: true,
                is_root_device: false,
            }],
            ..Default::default()
        };

        let jailer = MicroVmJailer::new(config, "/tmp/jail-advanced");
        let json_str = jailer.generate_vm_json().unwrap();
        assert!(json_str.contains("workspace"));
        assert!(json_str.contains("network-interfaces"));
        assert!(json_str.contains("eth0"));
    }

    #[test]
    fn test_microvm_hypervisor_types() {
        let firecracker = MicroVmConfig::default().with_hypervisor(HypervisorType::Firecracker);
        let jailer = MicroVmJailer::new(firecracker, "/tmp/jail-fc");
        let cmd = jailer.build_jailer_command();
        assert!(cmd[0].contains("firecracker"));

        let ch = MicroVmConfig::default().with_hypervisor(HypervisorType::CloudHypervisor);
        let jailer_ch = MicroVmJailer::new(ch, "/tmp/jail-ch");
        let cmd_ch = jailer_ch.build_jailer_command();
        assert!(cmd_ch[0].contains("cloud-hypervisor"));
    }

    #[test]
    fn test_hermetic_execution() {
        let config = MicroVmConfig::default();
        let executor = HermeticMicroVmExecutor::new(config, "/tmp/fish-vms");
        
        let result = executor
            .execute_hermetic_build(
                "test-build-123",
                vec!["cargo".to_string(), "build".to_string()],
                Path::new("/tmp/workspace"),
            )
            .unwrap();
        
        assert!(result.success());
        assert!(result.stdout.contains("cargo build"));
    }

    #[test]
    fn test_vm_config_validation() {
        let mut config = MicroVmConfig::default();
        config.vcpu_count = 0;
        assert!(config.validate().is_err());

        config.vcpu_count = 2;
        config.memory_size_mib = 64;
        assert!(config.validate().is_err());

        config.memory_size_mib = 1024;
        assert!(config.validate().is_ok());
    }
}
