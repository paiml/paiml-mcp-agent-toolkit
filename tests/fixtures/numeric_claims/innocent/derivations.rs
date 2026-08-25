// The ten hand-audited correct derivations from the attack corpus. Each
// annotation computes the value beside it out of parts, so the parts
// disagreeing with the whole is arithmetic, not contradiction.

const A_BYTES: usize = 1088;    // 64 rows * 17 bytes
const B_BYTES: usize = 1104;    // 64 * 17 + 16 bytes header
const C_BYTES: usize = 320;     // 4 lanes * 8 regs * 10 bytes
const D_MS: u64 = 250;          // 1000 ms / 4 workers
const E_BYTES: usize = 3072;    // 3 * 1024 bytes
const F_MS: u64 = 90;           // 30 ms * 3 retries
const G_BYTES: usize = 132;     // 128 bytes payload + 4 bytes crc
const H_MS: u64 = 1500;         // 500 ms budget, 3 phases
const I_BYTES: usize = 2080;    // (64 + 1) * 32 bytes
const J_MS: u64 = 7200000;      // 2 hours
