// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_macs_toolchain {
    use super::*;
    use tempfile::tempdir;

    fn rust_project() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        dir
    }

    fn write_toolchain(dir: &tempfile::TempDir, body: &str) {
        std::fs::write(dir.path().join("rust-toolchain.toml"), body).unwrap();
    }

    #[test]
    fn cb1667_skips_a_non_rust_project() {
        let dir = tempdir().unwrap();
        let check = check_toolchain_pinned(dir.path());
        assert_eq!(check.status, CheckStatus::Skip, "{}", check.message);
    }

    #[test]
    fn cb1667_fails_when_the_file_is_absent() {
        let dir = rust_project();
        let check = check_toolchain_pinned(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("rust-toolchain.toml"), "{}", check.message);
    }

    #[test]
    fn cb1667_passes_on_an_exact_version() {
        let dir = rust_project();
        write_toolchain(&dir, "[toolchain]\nchannel = \"1.98.1\"\n");
        let check = check_toolchain_pinned(dir.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cb1667_fails_on_a_floating_channel() {
        for channel in ["stable", "beta", "nightly"] {
            let dir = rust_project();
            write_toolchain(&dir, &format!("[toolchain]\nchannel = \"{channel}\"\n"));
            let check = check_toolchain_pinned(dir.path());
            assert_eq!(check.status, CheckStatus::Fail, "{channel}: {}", check.message);
        }
    }

    #[test]
    fn cb1667_passes_on_a_dated_nightly() {
        let dir = rust_project();
        write_toolchain(&dir, "[toolchain]\nchannel = \"nightly-2026-10-01\"\n");
        let check = check_toolchain_pinned(dir.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cb1667_fails_when_the_file_names_no_channel() {
        let dir = rust_project();
        write_toolchain(&dir, "[toolchain]\ncomponents = [\"clippy\"]\n");
        let check = check_toolchain_pinned(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
    }
}
