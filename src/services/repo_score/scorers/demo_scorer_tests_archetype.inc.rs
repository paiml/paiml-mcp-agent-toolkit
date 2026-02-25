    // RepoArchetype Tests - Coverage for all archetype methods

    #[test]
    fn test_repo_archetype_g2_max_scores() {
        // Test all archetypes for G2 max score values
        assert_eq!(RepoArchetype::Cookbook.g2_max_score(), None);
        assert_eq!(RepoArchetype::Tutorial.g2_max_score(), Some(1.5));
        assert_eq!(RepoArchetype::DemoApp.g2_max_score(), Some(3.0));
        assert_eq!(RepoArchetype::Library.g2_max_score(), Some(3.0));
        assert_eq!(RepoArchetype::Boilerplate.g2_max_score(), Some(3.0));
    }

    #[test]
    fn test_repo_archetype_names() {
        assert_eq!(RepoArchetype::Cookbook.name(), "Cookbook");
        assert_eq!(RepoArchetype::DemoApp.name(), "Demo Application");
        assert_eq!(RepoArchetype::Library.name(), "Library");
        assert_eq!(RepoArchetype::Tutorial.name(), "Tutorial");
        assert_eq!(RepoArchetype::Boilerplate.name(), "Boilerplate");
    }

    #[test]
    fn test_demo_scorer_default() {
        let scorer = DemoScorer::default();
        assert_eq!(scorer.category_name(), "Demo Quality");
        assert_eq!(scorer.max_score(), 10.0);
    }

    // Archetype Detection Tests - Coverage for detect_archetype

    #[tokio::test]
    async fn test_archetype_detection_boilerplate_by_name() {
        let temp_dir = TempDir::with_prefix("my-starter").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Boilerplate);
    }

    #[tokio::test]
    async fn test_archetype_detection_template_by_name() {
        let temp_dir = TempDir::with_prefix("rust-template").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Boilerplate);
    }

    #[tokio::test]
    async fn test_archetype_detection_scaffold_by_name() {
        let temp_dir =
            TempDir::with_prefix("project-scaffold").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Boilerplate);
    }

    #[tokio::test]
    async fn test_archetype_detection_tutorial_by_name() {
        let temp_dir = TempDir::with_prefix("rust-tutorial").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Tutorial);
    }

    #[tokio::test]
    async fn test_archetype_detection_learn_by_name() {
        let temp_dir = TempDir::with_prefix("learn-rust").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Tutorial);
    }

    #[tokio::test]
    async fn test_archetype_detection_course_by_name() {
        let temp_dir = TempDir::with_prefix("rust-course").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Tutorial);
    }

    #[tokio::test]
    async fn test_archetype_detection_recipes_by_name() {
        let temp_dir = TempDir::with_prefix("rust-recipes").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Cookbook);
    }

    #[tokio::test]
    async fn test_archetype_detection_demo_by_name() {
        let temp_dir = TempDir::with_prefix("my-demo").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::DemoApp);
    }

    #[tokio::test]
    async fn test_archetype_detection_example_by_name() {
        let temp_dir =
            TempDir::with_prefix("example-project").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::DemoApp);
    }

    #[tokio::test]
    async fn test_archetype_detection_cookbook_by_content() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create many markdown files but few code files
        for i in 0..10 {
            fs::write(repo_path.join(format!("doc{}.md", i)), "# Doc").expect("Write failed");
        }
        fs::write(repo_path.join("example.rs"), "fn main() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;
        assert_eq!(archetype, RepoArchetype::Cookbook);
    }

    #[tokio::test]
    async fn test_archetype_detection_demo_app_by_content() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create src/main.rs and demo files
        fs::create_dir_all(repo_path.join("src")).expect("Mkdir failed");
        fs::write(repo_path.join("src/main.rs"), "fn main() {}").expect("Write failed");
        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(repo_path.join("examples/demo.rs"), "fn main() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;
        assert_eq!(archetype, RepoArchetype::DemoApp);
    }

    #[tokio::test]
    async fn test_archetype_detection_library_with_both_main_and_lib() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create src/lib.rs AND src/main.rs - should be Library
        fs::create_dir_all(repo_path.join("src")).expect("Mkdir failed");
        fs::write(repo_path.join("src/lib.rs"), "pub fn hello() {}").expect("Write failed");
        fs::write(repo_path.join("src/main.rs"), "fn main() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;
        assert_eq!(archetype, RepoArchetype::Library);
    }

    #[tokio::test]
    async fn test_archetype_detection_with_src_but_no_lib_or_main() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create src/ dir but no lib.rs or main.rs
        fs::create_dir_all(repo_path.join("src")).expect("Mkdir failed");
        fs::write(repo_path.join("src/utils.rs"), "pub fn util() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;
        assert_eq!(archetype, RepoArchetype::Library);
    }
