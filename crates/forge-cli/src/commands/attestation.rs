use std::process::ExitCode;

use crate::args::{AttestArgs, VerifyArgs};
use crate::attestation;
use crate::utils::resolve_start_dir;

pub fn run_attest(args: AttestArgs) -> ExitCode {
    let start_dir = match resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let target_dir = start_dir.join("target");
    let mut outputs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                outputs.push(path);
            }
        }
    }

    match attestation::AttestationEngine::generate_attestation(&start_dir, &outputs) {
        Ok(attestation) => {
            let path = attestation::AttestationEngine::save_attestation(&start_dir, &attestation);
            match path {
                Ok(saved_path) => {
                    println!("🔒 SLSA Level 3 Attestation generated: {}", saved_path.display());
                    println!("   Merkle Root: {}", attestation.merkle_root);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: failed to save attestation: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            eprintln!("error: failed to generate attestation: {e}");
            ExitCode::FAILURE
        }
    }
}

pub fn run_verify(args: VerifyArgs) -> ExitCode {
    let start_dir = match resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let target_dir = start_dir.join("target");
    match attestation::AttestationEngine::verify_attestation(&args.attestation_file, &target_dir) {
        Ok(true) => {
            println!("✓ SLSA Provenance Verified: Artifacts are pristine and tamper-free.");
            ExitCode::SUCCESS
        }
        Ok(false) => {
            eprintln!("✗ SLSA Verification Failed: Artifacts modified or checksum mismatch!");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("error: verification failed: {e}");
            ExitCode::FAILURE
        }
    }
}
