// C++ test file
#include <iostream>

int main() {
    std::cout << "Hello C++" << std::endl;
    return 0;
}

int addNumbers(int a, int b) {
    return a + b;
}

int fibonacci(int n) {
    if (n <= 1)
        return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

class Calculator {
private:
    double value;

public:
    Calculator() : value(0.0) {}

    void calculate(char op, double val) {
        switch (op) {
            case '+':
                value += val;
                break;
            case '-':
                value -= val;
                break;
        }
    }

    double getValue() const {
        return value;
    }
};

template<typename T>
T max(T a, T b) {
    return (a > b) ? a : b;
}