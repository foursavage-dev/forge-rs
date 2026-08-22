pub fn compute_sha_preview(data: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for byte in data {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha_preview() {
        let h = compute_sha_preview(b"fish-wasm");
        assert_ne!(h, 0);
    }
}
