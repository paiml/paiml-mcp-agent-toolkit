fn innocent() {
    // let n = compute(4);
    let mask = 1 << 20;
    const CHUNK: usize = 4 * 1024 * 1024;
    let _ = (mask, CHUNK);
}
