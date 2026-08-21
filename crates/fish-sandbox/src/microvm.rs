use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroVmConfig {
    pub vcpu_count: u8,
    pub memory_size_mib: u32,
    pub kernel_image_path: PathBuf,
    pub rootfs_path: PathBuf,
    pub enable_network: bool,
    pub read_only_rootfs: bool,
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
        }
    }
}

pub struct MicroVmJailer {
    config: MicroVmConfig,
    jail_dir: PathBuf,
}

impl MicroVmJailer {
    pub fn new(config: MicroVmConfig, jail_dir: impl AsRef<Path>) -> Self {
        Self {
            config,
            jail_dir: jail_dir.as_ref().to_path_buf(),
        }
    }

    pub fn build_jailer_command(&self) -> Vec<String> {
        vec![
            "firecracker".to_string(),
            "--config-file".to_string(),
            self.jail_dir
                .join("vm_config.json")
                .to_string_lossy()
                .to_string(),
        ]
    }

    pub fn generate_vm_json(&self) -> Result<String, serde_json::Error> {
        let payload = serde_json::json!({
            "boot-source": {
                "kernel_image_path": self.config.kernel_image_path.to_string_lossy(),
                "boot_args": "console=ttyS0 reboot=k panic=1 pci=off nomodules rw init=/init"
            },
            "drives": [
                {
                    "drive_id": "rootfs",
                    "path_on_host": self.config.rootfs_path.to_string_lossy(),
                    "is_root_device": true,
                    "is_read_only": self.config.read_only_rootfs
                }
            ],
            "machine-config": {
                "vcpu_count": self.config.vcpu_count,
                "mem_size_mib": self.config.memory_size_mib
            }
        });
        serde_json::to_string_pretty(&payload)
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
}
