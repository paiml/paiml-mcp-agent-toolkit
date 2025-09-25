// TypeScript test file
function main(): void {
    console.log("Hello TypeScript");
}

function addNumbers(a: number, b: number): number {
    return a + b;
}

function fibonacci(n: number): number {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

class Calculator {
    private value: number = 0;

    calculate(op: string, val: number): void {
        switch (op) {
            case '+':
                this.value += val;
                break;
            case '-':
                this.value -= val;
                break;
        }
    }

    getValue(): number {
        return this.value;
    }
}

interface Point {
    x: number;
    y: number;
}

enum Color {
    Red,
    Green,
    Blue
}

type MathOperation = (a: number, b: number) => number;

const multiply: MathOperation = (a, b) => a * b;

main();