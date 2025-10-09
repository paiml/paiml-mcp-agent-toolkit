// test_calculator.cpp
// Google Test suite for Calculator mutation testing
#include <gtest/gtest.h>
#include "calculator.hpp"

// ============================================================
// ARITHMETIC OPERATOR TESTS (AOR)
// ============================================================

TEST(CalculatorTest, Add) {
    EXPECT_EQ(Calculator::Add(2, 3), 5);
    EXPECT_EQ(Calculator::Add(-1, -1), -2);
    EXPECT_EQ(Calculator::Add(0, 5), 5);
    EXPECT_EQ(Calculator::Add(-3, 5), 2);
    EXPECT_EQ(Calculator::Add(100, 200), 300);
}

TEST(CalculatorTest, Subtract) {
    EXPECT_EQ(Calculator::Subtract(5, 3), 2);
    EXPECT_EQ(Calculator::Subtract(-1, -1), 0);
    EXPECT_EQ(Calculator::Subtract(0, 5), -5);
    EXPECT_EQ(Calculator::Subtract(-3, 5), -8);
    EXPECT_EQ(Calculator::Subtract(10, 3), 7);
}

TEST(CalculatorTest, Multiply) {
    EXPECT_EQ(Calculator::Multiply(2, 3), 6);
    EXPECT_EQ(Calculator::Multiply(-2, 3), -6);
    EXPECT_EQ(Calculator::Multiply(0, 5), 0);
    EXPECT_EQ(Calculator::Multiply(-2, -3), 6);
    EXPECT_EQ(Calculator::Multiply(7, 8), 56);
}

TEST(CalculatorTest, Divide) {
    EXPECT_EQ(Calculator::Divide(6, 3), 2);
    EXPECT_EQ(Calculator::Divide(-6, 3), -2);
    EXPECT_EQ(Calculator::Divide(5, 2), 2); // Integer division
    EXPECT_EQ(Calculator::Divide(5, 0), 0); // Division by zero handling
    EXPECT_EQ(Calculator::Divide(20, 4), 5);
}

TEST(CalculatorTest, Modulo) {
    EXPECT_EQ(Calculator::Modulo(7, 3), 1);
    EXPECT_EQ(Calculator::Modulo(10, 5), 0);
    EXPECT_EQ(Calculator::Modulo(5, 2), 1);
    EXPECT_EQ(Calculator::Modulo(5, 0), 0); // Modulo by zero handling
    EXPECT_EQ(Calculator::Modulo(13, 4), 1);
}

// ============================================================
// RELATIONAL OPERATOR TESTS (ROR)
// ============================================================

TEST(CalculatorTest, GreaterThan) {
    EXPECT_TRUE(Calculator::GreaterThan(5, 3));
    EXPECT_FALSE(Calculator::GreaterThan(3, 5));
    EXPECT_FALSE(Calculator::GreaterThan(3, 3));
    EXPECT_TRUE(Calculator::GreaterThan(100, 50));
    EXPECT_FALSE(Calculator::GreaterThan(-5, 0));
}

TEST(CalculatorTest, LessThan) {
    EXPECT_TRUE(Calculator::LessThan(3, 5));
    EXPECT_FALSE(Calculator::LessThan(5, 3));
    EXPECT_FALSE(Calculator::LessThan(3, 3));
    EXPECT_TRUE(Calculator::LessThan(-10, 0));
    EXPECT_FALSE(Calculator::LessThan(100, 50));
}

TEST(CalculatorTest, GreaterOrEqual) {
    EXPECT_TRUE(Calculator::GreaterOrEqual(5, 3));
    EXPECT_TRUE(Calculator::GreaterOrEqual(3, 3));
    EXPECT_FALSE(Calculator::GreaterOrEqual(3, 5));
    EXPECT_TRUE(Calculator::GreaterOrEqual(10, 10));
    EXPECT_FALSE(Calculator::GreaterOrEqual(5, 10));
}

TEST(CalculatorTest, LessOrEqual) {
    EXPECT_TRUE(Calculator::LessOrEqual(3, 5));
    EXPECT_TRUE(Calculator::LessOrEqual(3, 3));
    EXPECT_FALSE(Calculator::LessOrEqual(5, 3));
    EXPECT_TRUE(Calculator::LessOrEqual(10, 10));
    EXPECT_FALSE(Calculator::LessOrEqual(15, 10));
}

TEST(CalculatorTest, Equal) {
    EXPECT_TRUE(Calculator::Equal(3, 3));
    EXPECT_FALSE(Calculator::Equal(3, 5));
    EXPECT_TRUE(Calculator::Equal(0, 0));
    EXPECT_TRUE(Calculator::Equal(-5, -5));
    EXPECT_FALSE(Calculator::Equal(10, 11));
}

TEST(CalculatorTest, NotEqual) {
    EXPECT_TRUE(Calculator::NotEqual(3, 5));
    EXPECT_FALSE(Calculator::NotEqual(3, 3));
    EXPECT_TRUE(Calculator::NotEqual(10, 20));
    EXPECT_FALSE(Calculator::NotEqual(-5, -5));
    EXPECT_TRUE(Calculator::NotEqual(0, 1));
}

// ============================================================
// LOGICAL OPERATOR TESTS (LOR)
// ============================================================

TEST(CalculatorTest, And) {
    EXPECT_TRUE(Calculator::And(true, true));
    EXPECT_FALSE(Calculator::And(true, false));
    EXPECT_FALSE(Calculator::And(false, true));
    EXPECT_FALSE(Calculator::And(false, false));
}

TEST(CalculatorTest, Or) {
    EXPECT_TRUE(Calculator::Or(true, true));
    EXPECT_TRUE(Calculator::Or(true, false));
    EXPECT_TRUE(Calculator::Or(false, true));
    EXPECT_FALSE(Calculator::Or(false, false));
}

// ============================================================
// BITWISE OPERATOR TESTS (BOR)
// ============================================================

TEST(CalculatorTest, BitwiseAnd) {
    EXPECT_EQ(Calculator::BitwiseAnd(5, 3), 1);   // 101 & 011 = 001
    EXPECT_EQ(Calculator::BitwiseAnd(12, 10), 8); // 1100 & 1010 = 1000
    EXPECT_EQ(Calculator::BitwiseAnd(15, 7), 7);  // 1111 & 0111 = 0111
    EXPECT_EQ(Calculator::BitwiseAnd(0, 255), 0);
}

TEST(CalculatorTest, BitwiseOr) {
    EXPECT_EQ(Calculator::BitwiseOr(5, 3), 7);    // 101 | 011 = 111
    EXPECT_EQ(Calculator::BitwiseOr(12, 10), 14); // 1100 | 1010 = 1110
    EXPECT_EQ(Calculator::BitwiseOr(8, 4), 12);   // 1000 | 0100 = 1100
    EXPECT_EQ(Calculator::BitwiseOr(0, 255), 255);
}

TEST(CalculatorTest, BitwiseXor) {
    EXPECT_EQ(Calculator::BitwiseXor(5, 3), 6);   // 101 ^ 011 = 110
    EXPECT_EQ(Calculator::BitwiseXor(12, 10), 6); // 1100 ^ 1010 = 0110
    EXPECT_EQ(Calculator::BitwiseXor(15, 15), 0); // Same values = 0
    EXPECT_EQ(Calculator::BitwiseXor(255, 0), 255);
}

TEST(CalculatorTest, LeftShift) {
    EXPECT_EQ(Calculator::LeftShift(5, 1), 10);  // 101 << 1 = 1010
    EXPECT_EQ(Calculator::LeftShift(3, 2), 12);  // 011 << 2 = 1100
    EXPECT_EQ(Calculator::LeftShift(1, 4), 16);  // 1 << 4 = 10000
    EXPECT_EQ(Calculator::LeftShift(7, 3), 56);
}

TEST(CalculatorTest, RightShift) {
    EXPECT_EQ(Calculator::RightShift(10, 1), 5); // 1010 >> 1 = 101
    EXPECT_EQ(Calculator::RightShift(12, 2), 3); // 1100 >> 2 = 011
    EXPECT_EQ(Calculator::RightShift(16, 4), 1); // 10000 >> 4 = 1
    EXPECT_EQ(Calculator::RightShift(100, 2), 25);
}

TEST(CalculatorTest, BitwiseNot) {
    EXPECT_EQ(Calculator::BitwiseNot(5), ~5);
    EXPECT_EQ(Calculator::BitwiseNot(0), -1);
    EXPECT_EQ(Calculator::BitwiseNot(-1), 0);
    EXPECT_EQ(Calculator::BitwiseNot(255), ~255);
}

// ============================================================
// UNARY OPERATOR TESTS (UOR)
// ============================================================

TEST(CalculatorTest, Not) {
    EXPECT_FALSE(Calculator::Not(true));
    EXPECT_TRUE(Calculator::Not(false));
}

TEST(CalculatorTest, Negate) {
    EXPECT_EQ(Calculator::Negate(5), -5);
    EXPECT_EQ(Calculator::Negate(-5), 5);
    EXPECT_EQ(Calculator::Negate(0), 0);
    EXPECT_EQ(Calculator::Negate(100), -100);
}

TEST(CalculatorTest, UnaryPlus) {
    EXPECT_EQ(Calculator::UnaryPlus(5), 5);
    EXPECT_EQ(Calculator::UnaryPlus(-5), -5);
    EXPECT_EQ(Calculator::UnaryPlus(0), 0);
}

TEST(CalculatorTest, PreIncrement) {
    EXPECT_EQ(Calculator::PreIncrement(5), 6);
    EXPECT_EQ(Calculator::PreIncrement(0), 1);
    EXPECT_EQ(Calculator::PreIncrement(-1), 0);
}

TEST(CalculatorTest, PostIncrement) {
    EXPECT_EQ(Calculator::PostIncrement(5), 5); // Returns original value
    EXPECT_EQ(Calculator::PostIncrement(0), 0);
    EXPECT_EQ(Calculator::PostIncrement(-1), -1);
}

TEST(CalculatorTest, PreDecrement) {
    EXPECT_EQ(Calculator::PreDecrement(5), 4);
    EXPECT_EQ(Calculator::PreDecrement(0), -1);
    EXPECT_EQ(Calculator::PreDecrement(1), 0);
}

TEST(CalculatorTest, PostDecrement) {
    EXPECT_EQ(Calculator::PostDecrement(5), 5); // Returns original value
    EXPECT_EQ(Calculator::PostDecrement(0), 0);
    EXPECT_EQ(Calculator::PostDecrement(1), 1);
}

// ============================================================
// POINTER OPERATOR TESTS (POR)
// ============================================================

TEST(CalculatorTest, Dereference) {
    int value = 42;
    int* ptr = &value;
    EXPECT_EQ(Calculator::Dereference(ptr), 42);

    value = 100;
    EXPECT_EQ(Calculator::Dereference(ptr), 100);

    // Null pointer handling
    EXPECT_EQ(Calculator::Dereference(nullptr), 0);
}

TEST(CalculatorTest, AddressOf) {
    int value = 42;
    int* ptr = Calculator::AddressOf(value);
    EXPECT_EQ(*ptr, 42);

    value = 100;
    EXPECT_EQ(*ptr, 100); // Pointer still points to same variable
}

// ============================================================
// MEMBER ACCESS TESTS (MAR)
// ============================================================

TEST(CalculatorTest, GetValue) {
    Calculator calc;
    calc.instanceValue = 100;
    EXPECT_EQ(calc.GetValue(), 100);

    calc.instanceValue = 42;
    EXPECT_EQ(calc.GetValue(), 42);
}

TEST(CalculatorTest, GetStaticValue) {
    EXPECT_EQ(Calculator::GetStaticValue(), 42);

    // Modify static value
    Calculator::staticValue = 100;
    EXPECT_EQ(Calculator::GetStaticValue(), 100);

    // Reset for other tests
    Calculator::staticValue = 42;
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
