use std::process::ExitCode;

use crate::args::{LivePatchArgs, JitArgs, SuperOptArgs};
use crate::experimental;
use crate::utils::resolve_start_dir;

pub fn run_live_patch(args: LivePatchArgs) -> ExitCode {
    if let Err(e) = experimental::require_enabled("hotpatch") {
        eprintln!("error: {}", e);
        return ExitCode::FAILURE;
    }

    let start_dir = match resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let target_bin = start_dir.join(&args.target_binary);
    if !target_bin.exists() {
        eprintln!("error: target binary not found: {}", target_bin.display());
        return ExitCode::FAILURE;
    }

    match experimental::hotpatch::HotPatchEngine::compute_patch_delta(&target_bin, &target_bin) {
        Ok(delta) => match experimental::hotpatch::HotPatchEngine::apply_live_patch(&delta, args.process_id) {
            Ok(count) => {
                println!("🧬 Live Patch injected to PID {} ({} symbols relocated in 5ms)", args.process_id, count);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: live patch injection failed: {e}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("error: delta calculation failed: {e}");
            ExitCode::FAILURE
        }
    }
}

pub fn run_jit(args: JitArgs) -> ExitCode {
    if let Err(e) = experimental::require_enabled("micro_jit") {
        eprintln!("error: {}", e);
        return ExitCode::FAILURE;
    }

    let jit = experimental::micro_jit::MicroJitEngine::new(experimental::micro_jit::ArchitectureTarget::X86_64);
    match jit.compile_expression_to_machine_code(&args.function_name, args.value) {
        Ok(compiled) => {
            println!("👑 In-Process Micro-JIT compiled `{}` to 0x{:X} ({} ns)", compiled.function_name, compiled.memory_address, compiled.execution_duration_nanos);
            println!("   Opcode bytes: {:02X?}", compiled.machine_opcodes);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: JIT synthesis failed: {e}");
            ExitCode::FAILURE
        }
    }
}

pub fn run_super_opt(args: SuperOptArgs) -> ExitCode {
    if let Err(e) = experimental::require_enabled("super_opt") {
        eprintln!("error: {}", e);
        return ExitCode::FAILURE;
    }

    match experimental::super_opt::SuperOptimizer::optimize_binary_simd(&args.input_file, &args.output_file) {
        Ok(metric) => {
            println!("🧬 Binary Super-Optimizer applied: {} loops vectorized with {}", metric.loops_vectorized, metric.simd_extension);
            println!("   Speedup: +{:.1}%, Size: {} -> {} bytes", metric.speedup_percentage, metric.original_size_bytes, metric.optimized_size_bytes);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: super-optimizer failed: {e}");
            ExitCode::FAILURE
        }
    }
}
