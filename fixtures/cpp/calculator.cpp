// calculator.cpp
// C++ Calculator - Implementation file for mutation testing
#include "calculator.hpp"

// Static member initialization
int Calculator::staticValue = 42;

// ============================================================
// ARITHMETIC OPERATORS (AOR)
// ============================================================

int Calculator::Add(int a, int b) {
    return a + b;
}

int Calculator::Subtract(int a, int b) {
    return a - b;
}

int Calculator::Multiply(int a, int b) {
    return a * b;
}

int Calculator::Divide(int a, int b) {
    if (b == 0) return 0; // Division by zero handling
    return a / b;
}

int Calculator::Modulo(int a, int b) {
    if (b == 0) return 0; // Modulo by zero handling
    return a % b;
}

// ============================================================
// RELATIONAL OPERATORS (ROR)
// ============================================================

bool Calculator::GreaterThan(int a, int b) {
    return a > b;
}

bool Calculator::LessThan(int a, int b) {
    return a < b;
}

bool Calculator::GreaterOrEqual(int a, int b) {
    return a >= b;
}

bool Calculator::LessOrEqual(int a, int b) {
    return a <= b;
}

bool Calculator::Equal(int a, int b) {
    return a == b;
}

bool Calculator::NotEqual(int a, int b) {
    return a != b;
}

// ============================================================
// LOGICAL OPERATORS (LOR)
// ============================================================

bool Calculator::And(bool a, bool b) {
    return a && b;
}

bool Calculator::Or(bool a, bool b) {
    return a || b;
}

// ============================================================
// BITWISE OPERATORS (BOR)
// ============================================================

int Calculator::BitwiseAnd(int a, int b) {
    return a & b;
}

int Calculator::BitwiseOr(int a, int b) {
    return a | b;
}

int Calculator::BitwiseXor(int a, int b) {
    return a ^ b;
}

int Calculator::LeftShift(int a, int shift) {
    return a << shift;
}

int Calculator::RightShift(int a, int shift) {
    return a >> shift;
}

int Calculator::BitwiseNot(int a) {
    return ~a;
}

// ============================================================
// UNARY OPERATORS (UOR)
// ============================================================

bool Calculator::Not(bool value) {
    return !value;
}

int Calculator::Negate(int value) {
    return -value;
}

int Calculator::UnaryPlus(int value) {
    return +value;
}

int Calculator::PreIncrement(int value) {
    return ++value;
}

int Calculator::PostIncrement(int value) {
    return value++;
}

int Calculator::PreDecrement(int value) {
    return --value;
}

int Calculator::PostDecrement(int value) {
    return value--;
}

// ============================================================
// POINTER OPERATORS (POR) - C++ SPECIFIC
// ============================================================

int Calculator::Dereference(int* ptr) {
    if (ptr == nullptr) return 0;
    return *ptr;
}

int* Calculator::AddressOf(int& value) {
    return &value;
}

// ============================================================
// MEMBER ACCESS (MAR) - C++ SPECIFIC
// ============================================================

int Calculator::GetValue() {
    return this->instanceValue;
}

int Calculator::GetStaticValue() {
    return Calculator::staticValue;
}
