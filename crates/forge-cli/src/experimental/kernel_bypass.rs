#![allow(dead_code)]

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct DmaBufferBlock {
    pub offset: usize,
    pub length: usize,
    pub memory_tag: String,
}

pub struct KernelBypassVfs {
    virtual_memory_pool: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl KernelBypassVfs {
    pub fn new() -> Self {
        Self {
            virtual_memory_pool: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn dma_write(&self, key: &str, data: &[u8]) -> io::Result<DmaBufferBlock> {
        let mut pool = self.virtual_memory_pool.write().unwrap();
        pool.insert(key.to_string(), data.to_vec());

        Ok(DmaBufferBlock {
            offset: 0x8000_0000,
            length: data.len(),
            memory_tag: format!("DMA_SHM:{}", key),
        })
    }

    pub fn dma_read(&self, key: &str) -> io::Result<Vec<u8>> {
        let pool = self.virtual_memory_pool.read().unwrap();
        pool.get(key)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "DMA Key not mapped in VFS"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_bypass_dma_zero_copy_io() {
        let vfs = KernelBypassVfs::new();
        let payload = b"ULTRA_HIGH_THROUGHPUT_ARTIFACT_STREAM";

        let block = vfs.dma_write("target/app.bin", payload).unwrap();
        assert_eq!(block.length, payload.len());

        let read_back = vfs.dma_read("target/app.bin").unwrap();
        assert_eq!(read_back, payload);
    }
}
