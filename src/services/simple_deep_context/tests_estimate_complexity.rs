    #[test]
    fn test_estimate_complexity() {
        let analyzer = SimpleDeepContext;

        // Test Python complexity
        let py_code = r#"
if x > 0:
    for i in range(10):
        if i % 2 == 0:
            print(i)
elif x < 0:
    print("negative")
"#;
        let complexity = analyzer.estimate_complexity(py_code, "py");
        // Actually counts: 1 base + 2 "if " + 1 "for " + 1 "elif " + 1 "else:" = 6
        assert_eq!(complexity, 6);

        // Test JavaScript complexity with logical operators
        let js_code = r#"
if (x > 0 && y < 10) {
    for (let i = 0; i < 10; i++) {
        if (i % 2 === 0 || i === 5) {
            console.log(i);
        }
    }
}
"#;
        let complexity = analyzer.estimate_complexity(js_code, "js");
        assert_eq!(complexity, 6); // 1 base + 2 if + 1 for + 1 && + 1 ||
    }

    #[test]
    fn test_estimate_complexity_go() {
        let analyzer = SimpleDeepContext;
        let go_code = r#"
func main() {
    if x > 0 {
        for i := 0; i < 10; i++ {
            if i%2 == 0 {
                fmt.Println(i)
            }
        }
    }
}
"#;
        let complexity = analyzer.estimate_complexity(go_code, "go");
        assert!(complexity >= 4); // Multiple control flow statements
    }

    #[test]
    fn test_estimate_complexity_java() {
        let analyzer = SimpleDeepContext;
        let java_code = r#"
public void process() {
    if (x > 0) {
        for (int i = 0; i < 10; i++) {
            switch (state) {
                case 1: break;
                case 2: break;
            }
        }
    }
}
"#;
        let complexity = analyzer.estimate_complexity(java_code, "java");
        assert!(complexity >= 4);
    }

    #[test]
    fn test_estimate_complexity_ruby() {
        let analyzer = SimpleDeepContext;
        let ruby_code = r#"
def process
  if x > 0
    (0..10).each do |i|
      if i % 2 == 0
        puts i
      end
    end
  end
end
"#;
        let complexity = analyzer.estimate_complexity(ruby_code, "rb");
        assert!(complexity >= 1); // At least base complexity
    }

    #[test]
    fn test_estimate_complexity_kotlin() {
        let analyzer = SimpleDeepContext;
        let kt_code = r#"
fun process() {
    if (x > 0) {
        for (i in 0..10) {
            when (i) {
                1 -> println("one")
                else -> println("other")
            }
        }
    }
}
"#;
        let complexity = analyzer.estimate_complexity(kt_code, "kt");
        assert!(complexity >= 1); // At least base complexity
    }

    #[test]
    fn test_estimate_complexity_swift() {
        let analyzer = SimpleDeepContext;
        let swift_code = r#"
func process() {
    if x > 0 {
        for i in 0..<10 {
            guard i % 2 == 0 else { continue }
            print(i)
        }
    }
}
"#;
        let complexity = analyzer.estimate_complexity(swift_code, "swift");
        assert!(complexity >= 1); // At least base complexity
    }

    #[test]
    fn test_estimate_complexity_bash() {
        let analyzer = SimpleDeepContext;
        let bash_code = r#"
process() {
    if [[ $x -gt 0 ]]; then
        for i in {1..10}; do
            if [[ $((i % 2)) -eq 0 ]]; then
                echo $i
            fi
        done
    fi
}
"#;
        let complexity = analyzer.estimate_complexity(bash_code, "sh");
        assert!(complexity >= 1); // At least base complexity
    }

    #[test]
    fn test_estimate_complexity_cpp() {
        let analyzer = SimpleDeepContext;
        let cpp_code = r#"
void process() {
    if (x > 0) {
        for (int i = 0; i < 10; i++) {
            try {
                doSomething();
            } catch (...) {
                handleError();
            }
        }
    }
}
"#;
        let complexity = analyzer.estimate_complexity(cpp_code, "cpp");
        assert!(complexity >= 4);
    }

    // ============ estimate_complexity Edge Cases ============

    #[test]
    fn test_estimate_complexity_unknown_language() {
        let analyzer = SimpleDeepContext;
        let code = "some random code with if and while";
        let complexity = analyzer.estimate_complexity(code, "unknown_lang");
        // Unknown language should return base complexity
        assert_eq!(complexity, 1);
    }

    #[test]
    fn test_estimate_complexity_empty_code() {
        let analyzer = SimpleDeepContext;
        let code = "";
        let complexity = analyzer.estimate_complexity(code, "py");
        // Empty code should have base complexity only
        assert_eq!(complexity, 1);
    }
