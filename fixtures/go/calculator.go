package calculator

// Add returns the sum of two integers
func Add(a, b int) int {
	return a + b
}

// Subtract returns the difference of two integers
func Subtract(a, b int) int {
	return a - b
}

// Multiply returns the product of two integers
func Multiply(a, b int) int {
	return a * b
}

// Divide returns the quotient of two integers
func Divide(a, b int) int {
	if b == 0 {
		panic("division by zero")
	}
	return a / b
}

// Modulo returns the remainder of division
func Modulo(a, b int) int {
	if b == 0 {
		panic("modulo by zero")
	}
	return a % b
}

// IsPositive checks if a number is positive
func IsPositive(value int) bool {
	return value > 0
}

// IsGreaterOrEqual checks if a >= b
func IsGreaterOrEqual(a, b int) bool {
	return a >= b
}

// IsEqual checks if two values are equal
func IsEqual(a, b int) bool {
	return a == b
}

// BothPositive checks if both numbers are positive
func BothPositive(a, b int) bool {
	return a > 0 && b > 0
}

// EitherPositive checks if at least one number is positive
func EitherPositive(a, b int) bool {
	return a > 0 || b > 0
}

// BitwiseAnd performs bitwise AND
func BitwiseAnd(a, b int) int {
	return a & b
}

// BitwiseOr performs bitwise OR
func BitwiseOr(a, b int) int {
	return a | b
}

// BitwiseXor performs bitwise XOR
func BitwiseXor(a, b int) int {
	return a ^ b
}

// LeftShift performs left shift
func LeftShift(a, shift int) int {
	return a << shift
}

// RightShift performs right shift
func RightShift(a, shift int) int {
	return a >> shift
}

// Negate negates a number
func Negate(value int) int {
	return -value
}

// Positive returns positive value (unary +)
func Positive(value int) int {
	return +value
}

// Not returns logical NOT
func Not(flag bool) bool {
	return !flag
}

// AddAssign demonstrates += operator
func AddAssign(value, delta int) int {
	value += delta
	return value
}

// SubtractAssign demonstrates -= operator
func SubtractAssign(value, delta int) int {
	value -= delta
	return value
}

// MultiplyAssign demonstrates *= operator
func MultiplyAssign(value, factor int) int {
	value *= factor
	return value
}

// DivideAssign demonstrates /= operator
func DivideAssign(value, divisor int) int {
	if divisor == 0 {
		panic("division by zero")
	}
	value /= divisor
	return value
}
