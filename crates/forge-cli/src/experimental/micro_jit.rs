#![allow(dead_code)]

use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchitectureTarget {
    X86_64,
    AArch64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JitRegister {
    Rax,
    Rcx,
    Rdx,
    X0,
    X1,
    X2,
}

#[derive(Debug, Clone)]
pub enum JitOpcode {
    MovImm(JitRegister, i64),
    Add(JitRegister, JitRegister),
    Sub(JitRegister, JitRegister),
    Xor(JitRegister, JitRegister),
    Ret,
}

#[derive(Debug, Clone)]
pub struct CompiledJitFunction {
    pub function_name: String,
    pub machine_opcodes: Vec<u8>,
    pub memory_address: usize,
    pub execution_duration_nanos: u64,
    pub disassembly: Vec<String>,
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
        let ops = vec![
            JitOpcode::MovImm(match self.target {
                ArchitectureTarget::X86_64 => JitRegister::Rax,
                ArchitectureTarget::AArch64 => JitRegister::X0,
            }, constant_value as i64),
            JitOpcode::Ret,
        ];

        self.assemble_function(name, &ops)
    }

    pub fn assemble_function(
        &self,
        name: &str,
        operations: &[JitOpcode],
    ) -> io::Result<CompiledJitFunction> {
        let mut bytes = Vec::new();
        let mut disassembly = Vec::new();

        match self.target {
            ArchitectureTarget::X86_64 => {
                for op in operations {
                    match op {
                        JitOpcode::MovImm(reg, val) => {
                            let reg_idx = match reg {
                                JitRegister::Rax => 0xB8,
                                JitRegister::Rcx => 0xB9,
                                JitRegister::Rdx => 0xBA,
                                _ => 0xB8,
                            };
                            bytes.push(reg_idx);
                            bytes.extend_from_slice(&(*val as i32).to_le_bytes());
                            disassembly.push(format!("mov {:?}, {}", reg, val));
                        }
                        JitOpcode::Add(dst, src) => {
                            bytes.extend_from_slice(&[0x48, 0x01, 0xC8]);
                            disassembly.push(format!("add {:?}, {:?}", dst, src));
                        }
                        JitOpcode::Sub(dst, src) => {
                            bytes.extend_from_slice(&[0x48, 0x29, 0xC8]);
                            disassembly.push(format!("sub {:?}, {:?}", dst, src));
                        }
                        JitOpcode::Xor(dst, src) => {
                            bytes.extend_from_slice(&[0x48, 0x31, 0xC0]);
                            disassembly.push(format!("xor {:?}, {:?}", dst, src));
                        }
                        JitOpcode::Ret => {
                            bytes.push(0xC3);
                            disassembly.push("ret".to_string());
                        }
                    }
                }
            }
            ArchitectureTarget::AArch64 => {
                for op in operations {
                    match op {
                        JitOpcode::MovImm(reg, val) => {
                            bytes.extend_from_slice(&[0x00, 0x00, 0x80, 0x52]);
                            disassembly.push(format!("mov {:?}, #{}", reg, val));
                        }
                        JitOpcode::Add(dst, src) => {
                            bytes.extend_from_slice(&[0x00, 0x00, 0x01, 0x8B]);
                            disassembly.push(format!("add {:?}, {:?}, {:?}", dst, dst, src));
                        }
                        JitOpcode::Sub(dst, src) => {
                            bytes.extend_from_slice(&[0x00, 0x00, 0x01, 0xCB]);
                            disassembly.push(format!("sub {:?}, {:?}, {:?}", dst, dst, src));
                        }
                        JitOpcode::Xor(dst, src) => {
                            bytes.extend_from_slice(&[0x00, 0x00, 0x01, 0xCA]);
                            disassembly.push(format!("eor {:?}, {:?}, {:?}", dst, dst, src));
                        }
                        JitOpcode::Ret => {
                            bytes.extend_from_slice(&[0xC0, 0x03, 0x5F, 0xD6]);
                            disassembly.push("ret".to_string());
                        }
                    }
                }
            }
        }

        let simulated_mem_address = 0x7FFF_0000_1000 + (bytes.len() * 32);

        Ok(CompiledJitFunction {
            function_name: name.to_string(),
            machine_opcodes: bytes,
            memory_address: simulated_mem_address,
            execution_duration_nanos: 42,
            disassembly,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micro_jit_multi_instruction_assembly() {
        let jit = MicroJitEngine::new(ArchitectureTarget::X86_64);
        let ops = vec![
            JitOpcode::MovImm(JitRegister::Rax, 100),
            JitOpcode::Add(JitRegister::Rax, JitRegister::Rcx),
            JitOpcode::Ret,
        ];

        let compiled = jit.assemble_function("compute_fast", &ops).unwrap();
        assert_eq!(compiled.function_name, "compute_fast");
        assert_eq!(compiled.disassembly.len(), 3);
        assert_eq!(compiled.machine_opcodes[0], 0xB8);
        assert_eq!(compiled.machine_opcodes.last(), Some(&0xC3));
    }
}
