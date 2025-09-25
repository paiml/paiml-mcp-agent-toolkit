// JavaScript test file
function main() {
    console.log("Hello JavaScript");
}

function addNumbers(a, b) {
    return a + b;
}

function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

class Calculator {
    constructor() {
        this.value = 0;
    }

    calculate(op, val) {
        switch (op) {
            case '+':
                this.value += val;
                break;
            case '-':
                this.value -= val;
                break;
        }
    }

    getValue() {
        return this.value;
    }
}

const multiply = (a, b) => a * b;

const mathUtils = {
    isPrime(n) {
        if (n <= 1) return false;
        for (let i = 2; i < n; i++) {
            if (n % i === 0) return false;
        }
        return true;
    },

    factorial(n) {
        if (n <= 1) return 1;
        return n * this.factorial(n - 1);
    }
};

main();