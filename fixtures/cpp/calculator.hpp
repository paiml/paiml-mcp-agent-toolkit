// calculator.hpp
// C++ Calculator - Header file for mutation testing
// All mutation operator types for comprehensive testing
#ifndef CALCULATOR_HPP
#define CALCULATOR_HPP

class Calculator {
public:
    // ============================================================
    // ARITHMETIC OPERATORS (AOR)
    // ============================================================
    static int Add(int a, int b);
    static int Subtract(int a, int b);
    static int Multiply(int a, int b);
    static int Divide(int a, int b);
    static int Modulo(int a, int b);

    // ============================================================
    // RELATIONAL OPERATORS (ROR)
    // ============================================================
    static bool GreaterThan(int a, int b);
    static bool LessThan(int a, int b);
    static bool GreaterOrEqual(int a, int b);
    static bool LessOrEqual(int a, int b);
    static bool Equal(int a, int b);
    static bool NotEqual(int a, int b);

    // ============================================================
    // LOGICAL OPERATORS (LOR)
    // ============================================================
    static bool And(bool a, bool b);
    static bool Or(bool a, bool b);

    // ============================================================
    // BITWISE OPERATORS (BOR)
    // ============================================================
    static int BitwiseAnd(int a, int b);
    static int BitwiseOr(int a, int b);
    static int BitwiseXor(int a, int b);
    static int LeftShift(int a, int shift);
    static int RightShift(int a, int shift);
    static int BitwiseNot(int a);

    // ============================================================
    // UNARY OPERATORS (UOR)
    // ============================================================
    static bool Not(bool value);
    static int Negate(int value);
    static int UnaryPlus(int value);
    static int PreIncrement(int value);
    static int PostIncrement(int value);
    static int PreDecrement(int value);
    static int PostDecrement(int value);

    // ============================================================
    // POINTER OPERATORS (POR) - C++ SPECIFIC
    // ============================================================
    static int Dereference(int* ptr);
    static int* AddressOf(int& value);

    // ============================================================
    // MEMBER ACCESS (MAR) - C++ SPECIFIC
    // ============================================================
    int instanceValue;
    static int staticValue;
    int GetValue();
    static int GetStaticValue();
};

#endif // CALCULATOR_HPP
