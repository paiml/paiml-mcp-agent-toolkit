// Tools handlers - split for file health (CB-040)
include!("core_tools.rs");
include!("extended_tools.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    // Tests from original file
}
