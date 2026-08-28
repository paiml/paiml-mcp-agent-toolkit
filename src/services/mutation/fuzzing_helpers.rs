/// Execute mutant code with given input (simulated for Phase 1)
/// Real implementation would compile mutant and execute with input
fn execute_mutant_with_input(_mutant_source: &str, input: &[u8]) -> Result<()> {
    // Phase 1: Simulate execution
    // This would be replaced with actual compilation + execution

    // Simulate crash on certain patterns
    if input.len() > 100 {
        // Simulate out-of-bounds access crash
        anyhow::bail!("Simulated crash: buffer overflow");
    }

    // Simulate success
    Ok(())
}

/// Mutate an input to create new test cases
fn mutate_input(seed: &[u8]) -> Vec<u8> {
    use rand::RngExt;
    let mut rng = rand::rng();
    let mut mutated = seed.to_vec();

    if mutated.is_empty() {
        return vec![rng.random::<u8>()];
    }

    // Apply random mutation strategy
    match rng.random_range(0..5) {
        0 => {
            // Bit flip
            if !mutated.is_empty() {
                let idx = rng.random_range(0..mutated.len());
                let bit = rng.random_range(0..8);
                mutated[idx] ^= 1 << bit;
            }
        }
        1 => {
            // Byte flip
            if !mutated.is_empty() {
                let idx = rng.random_range(0..mutated.len());
                mutated[idx] = rng.random::<u8>();
            }
        }
        2 => {
            // Insert byte
            let idx = rng.random_range(0..=mutated.len());
            mutated.insert(idx, rng.random::<u8>());
        }
        3 => {
            // Delete byte.
            //
            // `> 1`, not `!is_empty()`: deleting the only byte of a 1-byte
            // seed returns an empty input, which contradicts this function's
            // own contract -- the empty-seed branch above deliberately hands
            // back a fresh byte rather than nothing -- and feeds a degenerate
            // entry straight back into the coverage-guided corpus in
            // `fuzzing_strategy.rs`. Deletion is 1 of 5 strategies chosen
            // uniformly, so a 1-byte seed came back empty 20% of the time;
            // that is why `test_mutate_input_single_byte` failed on the second
            // run of the `full` suite and passed on the first. libFuzzer's
            // EraseBytes has the same floor for the same reason.
            if mutated.len() > 1 {
                let idx = rng.random_range(0..mutated.len());
                mutated.remove(idx);
            }
        }
        4 => {
            // Append bytes
            let count = rng.random_range(1..=4);
            for _ in 0..count {
                mutated.push(rng.random::<u8>());
            }
        }
        _ => unreachable!(),
    }

    mutated
}
