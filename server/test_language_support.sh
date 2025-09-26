#!/bin/bash
# Test script to verify language support implementation

echo "=== Language Support Verification ==="
echo

# Create test directory
TEST_DIR="/tmp/pmat_lang_test_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Create test files for each language
cat > test.cs <<'EOF'
using System;
namespace TestApp {
    class Program {
        static void Main(string[] args) {
            Console.WriteLine("C# Test");
        }
        static int Calculate(int x) {
            if (x > 10) return x * 2;
            return x + 5;
        }
    }
}
EOF

cat > test.go <<'EOF'
package main
import "fmt"
func main() {
    fmt.Println("Go Test")
}
func ProcessData(data []int) int {
    result := 0
    for _, val := range data {
        result += val
    }
    return result
}
EOF

cat > Test.java <<'EOF'
public class Test {
    public static void main(String[] args) {
        System.out.println("Java Test");
    }
    public int calculate(int a, int b) {
        if (a > b) {
            return a * b;
        }
        return a + b;
    }
}
EOF

cat > test.kt <<'EOF'
fun main() {
    println("Kotlin Test")
}
fun processNumbers(numbers: List<Int>): Int {
    return numbers.sum()
}
EOF

cat > test.rb <<'EOF'
def greet(name)
  puts "Hello, #{name}!"
end
def calculate_grade(score)
  return "A" if score >= 90
  return "B" if score >= 80
  "C"
end
EOF

echo "Test files created in $TEST_DIR:"
ls -la

echo
echo "Running pmat context analysis..."

# Try with installed version first
if command -v pmat &> /dev/null; then
    echo "Using installed pmat $(pmat --version)"
    pmat context --output deep_context_test.md 2>&1 | head -30

    echo
    echo "=== Checking language detection ==="
    echo

    if [ -f deep_context_test.md ]; then
        echo "C# Detection:"
        grep -A5 "test.cs" deep_context_test.md | head -10

        echo
        echo "Go Detection:"
        grep -A5 "test.go" deep_context_test.md | head -10

        echo
        echo "Java Detection:"
        grep -A5 "Test.java" deep_context_test.md | head -10

        echo
        echo "Kotlin Detection:"
        grep -A5 "test.kt" deep_context_test.md | head -10

        echo
        echo "Ruby Detection:"
        grep -A5 "test.rb" deep_context_test.md | head -10
    else
        echo "ERROR: deep_context_test.md not generated"
    fi
else
    echo "pmat not found in PATH"
fi

# Cleanup
cd /
rm -rf "$TEST_DIR"

echo
echo "=== Test Complete ==="