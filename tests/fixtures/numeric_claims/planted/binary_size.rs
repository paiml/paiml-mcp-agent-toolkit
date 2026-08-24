// Planted defect 2 of 4 — C5 NAMED CROSS-REFERENCE.
//
// The declaration says it is aligned with a key that holds a different number.
// 50 * 1024 * 1024 is 52,428,800; binary_max_bytes is 50,000,000.

fn threshold() -> u64 {
    const MAX_SIZE_BYTES: u64 = 50 * 1024 * 1024; // 50MB (aligned with .pmat-metrics.toml binary_max_bytes)
    MAX_SIZE_BYTES
}
