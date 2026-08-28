#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_collect_assemblyscript_files() {
        let temp_dir = TempDir::new().unwrap();
        let as_file = temp_dir.path().join("test.as");
        let ts_file = temp_dir.path().join("assembly.ts");
        let other_file = temp_dir.path().join("test.txt");

        tokio::fs::write(&as_file, "function test(): i32 { return 42; }")
            .await
            .unwrap();
        tokio::fs::write(&ts_file, "const value: i32 = 42; @global let ptr: usize;")
            .await
            .unwrap();
        tokio::fs::write(&other_file, "not assemblyscript")
            .await
            .unwrap();

        let files = collect_assemblyscript_files(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_collect_wasm_files() {
        let temp_dir = TempDir::new().unwrap();
        let wasm_file = temp_dir.path().join("test.wasm");
        let wat_file = temp_dir.path().join("test.wat");
        let other_file = temp_dir.path().join("test.txt");

        tokio::fs::write(&wasm_file, b"\0asm\x01\x00\x00\x00")
            .await
            .unwrap();
        tokio::fs::write(&wat_file, "(module)").await.unwrap();
        tokio::fs::write(&other_file, "not wasm").await.unwrap();

        let files = collect_wasm_files(temp_dir.path(), true, true).unwrap();
        assert_eq!(files.len(), 2);
    }

    /// Both collectors walked the tree with a bare `WalkDir` that read no
    /// `.gitignore`. Run against pmat's own repository, `analyze
    /// assembly-script` reported 48 files: one `.ts` file, once per checkout
    /// under the gitignored `.claude/worktrees/`.
    ///
    /// The fixture uses a plain-named ignored directory on purpose — skipping
    /// only hidden entries would hide the defect rather than fix it.
    #[tokio::test]
    async fn collectors_skip_a_gitignored_copy_of_the_project() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        tokio::fs::write(root.join(".gitignore"), "worktrees/\n")
            .await
            .unwrap();
        tokio::fs::write(root.join("assembly.ts"), "const value: i32 = 42;")
            .await
            .unwrap();
        tokio::fs::write(root.join("module.wat"), "(module)")
            .await
            .unwrap();

        for copy in ["worktrees/a", "worktrees/b"] {
            tokio::fs::create_dir_all(root.join(copy)).await.unwrap();
            tokio::fs::write(root.join(copy).join("assembly.ts"), "const value: i32 = 42;")
                .await
                .unwrap();
            tokio::fs::write(root.join(copy).join("module.wat"), "(module)")
                .await
                .unwrap();
        }

        let as_files = collect_assemblyscript_files(root).unwrap();
        assert_eq!(
            as_files.len(),
            1,
            "one AssemblyScript file is part of the project, not three: {as_files:?}"
        );

        let wasm_files = collect_wasm_files(root, true, true).unwrap();
        assert_eq!(
            wasm_files.len(),
            1,
            "one .wat file is part of the project, not three: {wasm_files:?}"
        );
    }
}
