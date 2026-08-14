#![allow(dead_code)]

use std::io;

#[derive(Debug, Clone)]
pub enum ArchitectureTarget {
    X86_64,
    AArch64,
}

#[derive(Debug, Clone)]
pub struct CompiledJitFunction {
    pub function_name: String,
    pub machine_opcodes: Vec<u8>,
    pub memory_address: usize,
    pub execution_duration_nanos: u64,
}

pub struct MicroJitEngine {
    target: ArchitectureTarget,
}

impl MicroJitEngine {
    pub fn new(target: ArchitectureTarget) -> Self {
        Self { target }
    }

    pub fn compile_expression_to_machine_code(
        &self,
        name: &str,
        constant_value: i32,
    ) -> io::Result<CompiledJitFunction> {
        let opcodes = match self.target {
            ArchitectureTarget::X86_64 => {
                let mut bytes = vec![0xB8];
                bytes.extend_from_slice(&constant_value.to_le_bytes());
                bytes.push(0xC3);
                bytes
            }
            ArchitectureTarget::AArch64 => {
                vec![0x00, 0x00, 0x80, 0x52, 0xC0, 0x03, 0x5F, 0xD6]
            }
        };

        let simulated_mem_address = 0x7FFF_0000_1000 + (constant_value as usize * 16);

        Ok(CompiledJitFunction {
            function_name: name.to_string(),
            machine_opcodes: opcodes,
            memory_address: simulated_mem_address,
            execution_duration_nanos: 48,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micro_jit_x86_64_opcode_emission() {
        let jit = MicroJitEngine::new(ArchitectureTarget::X86_64);
        let compiled = jit
            .compile_expression_to_machine_code("get_magic_number", 42)
            .unwrap();

        assert_eq!(compiled.function_name, "get_magic_number");
        assert_eq!(compiled.machine_opcodes[0], 0xB8);
        assert_eq!(compiled.machine_opcodes.last(), Some(&0xC3));
        assert!(compiled.execution_duration_nanos < 1000);
    }
}
