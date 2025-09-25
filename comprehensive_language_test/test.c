// C test file
#include <stdio.h>

int main() {
    printf("Hello C\n");
    return 0;
}

int add_numbers(int a, int b) {
    return a + b;
}

int fibonacci(int n) {
    if (n <= 1)
        return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

typedef struct {
    double value;
} Calculator;

Calculator* new_calculator() {
    Calculator* calc = malloc(sizeof(Calculator));
    calc->value = 0.0;
    return calc;
}

void calculate(Calculator* calc, char op, double val) {
    switch (op) {
        case '+':
            calc->value += val;
            break;
        case '-':
            calc->value -= val;
            break;
    }
}