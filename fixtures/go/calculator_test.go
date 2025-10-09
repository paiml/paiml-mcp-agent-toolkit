package calculator

import "testing"

func TestAdd(t *testing.T) {
	tests := []struct {
		name     string
		a, b     int
		expected int
	}{
		{"positive numbers", 2, 3, 5},
		{"negative numbers", -1, -1, -2},
		{"zero", 0, 5, 5},
		{"mixed", -3, 5, 2},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := Add(tt.a, tt.b)
			if result != tt.expected {
				t.Errorf("Add(%d, %d) = %d; want %d", tt.a, tt.b, result, tt.expected)
			}
		})
	}
}

func TestSubtract(t *testing.T) {
	tests := []struct {
		a, b     int
		expected int
	}{
		{5, 3, 2},
		{10, 5, 5},
		{0, 5, -5},
		{-3, -3, 0},
	}

	for _, tt := range tests {
		if result := Subtract(tt.a, tt.b); result != tt.expected {
			t.Errorf("Subtract(%d, %d) = %d; want %d", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestMultiply(t *testing.T) {
	tests := []struct {
		a, b     int
		expected int
	}{
		{4, 5, 20},
		{0, 10, 0},
		{-2, 3, -6},
		{-2, -3, 6},
	}

	for _, tt := range tests {
		if result := Multiply(tt.a, tt.b); result != tt.expected {
			t.Errorf("Multiply(%d, %d) = %d; want %d", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestDivide(t *testing.T) {
	tests := []struct {
		a, b     int
		expected int
	}{
		{10, 2, 5},
		{20, 4, 5},
		{15, 3, 5},
		{-10, 2, -5},
	}

	for _, tt := range tests {
		if result := Divide(tt.a, tt.b); result != tt.expected {
			t.Errorf("Divide(%d, %d) = %d; want %d", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestModulo(t *testing.T) {
	tests := []struct {
		a, b     int
		expected int
	}{
		{10, 3, 1},
		{15, 4, 3},
		{20, 5, 0},
		{7, 3, 1},
	}

	for _, tt := range tests {
		if result := Modulo(tt.a, tt.b); result != tt.expected {
			t.Errorf("Modulo(%d, %d) = %d; want %d", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestIsPositive(t *testing.T) {
	tests := []struct {
		value    int
		expected bool
	}{
		{5, true},
		{0, false},
		{-5, false},
		{100, true},
		{-1, false},
	}

	for _, tt := range tests {
		if result := IsPositive(tt.value); result != tt.expected {
			t.Errorf("IsPositive(%d) = %v; want %v", tt.value, result, tt.expected)
		}
	}
}

func TestIsGreaterOrEqual(t *testing.T) {
	tests := []struct {
		a, b     int
		expected bool
	}{
		{5, 3, true},
		{3, 3, true},
		{3, 5, false},
		{0, 0, true},
		{-1, 1, false},
	}

	for _, tt := range tests {
		if result := IsGreaterOrEqual(tt.a, tt.b); result != tt.expected {
			t.Errorf("IsGreaterOrEqual(%d, %d) = %v; want %v", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestIsEqual(t *testing.T) {
	tests := []struct {
		a, b     int
		expected bool
	}{
		{5, 5, true},
		{5, 3, false},
		{0, 0, true},
		{-1, -1, true},
		{-1, 1, false},
	}

	for _, tt := range tests {
		if result := IsEqual(tt.a, tt.b); result != tt.expected {
			t.Errorf("IsEqual(%d, %d) = %v; want %v", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestBothPositive(t *testing.T) {
	tests := []struct {
		a, b     int
		expected bool
	}{
		{5, 10, true},
		{5, -1, false},
		{-1, 5, false},
		{-1, -1, false},
		{0, 5, false},
		{5, 0, false},
	}

	for _, tt := range tests {
		if result := BothPositive(tt.a, tt.b); result != tt.expected {
			t.Errorf("BothPositive(%d, %d) = %v; want %v", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestEitherPositive(t *testing.T) {
	tests := []struct {
		a, b     int
		expected bool
	}{
		{5, 10, true},
		{5, -1, true},
		{-1, 5, true},
		{-1, -1, false},
		{0, 0, false},
	}

	for _, tt := range tests {
		if result := EitherPositive(tt.a, tt.b); result != tt.expected {
			t.Errorf("EitherPositive(%d, %d) = %v; want %v", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestBitwiseAnd(t *testing.T) {
	tests := []struct {
		a, b     int
		expected int
	}{
		{6, 3, 2},   // 110 & 011 = 010
		{12, 10, 8}, // 1100 & 1010 = 1000
		{15, 7, 7},  // 1111 & 0111 = 0111
	}

	for _, tt := range tests {
		if result := BitwiseAnd(tt.a, tt.b); result != tt.expected {
			t.Errorf("BitwiseAnd(%d, %d) = %d; want %d", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestBitwiseOr(t *testing.T) {
	tests := []struct {
		a, b     int
		expected int
	}{
		{6, 3, 7},   // 110 | 011 = 111
		{12, 10, 14}, // 1100 | 1010 = 1110
		{8, 4, 12},  // 1000 | 0100 = 1100
	}

	for _, tt := range tests {
		if result := BitwiseOr(tt.a, tt.b); result != tt.expected {
			t.Errorf("BitwiseOr(%d, %d) = %d; want %d", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestBitwiseXor(t *testing.T) {
	tests := []struct {
		a, b     int
		expected int
	}{
		{6, 3, 5},   // 110 ^ 011 = 101
		{12, 10, 6}, // 1100 ^ 1010 = 0110
		{15, 15, 0}, // Same values = 0
	}

	for _, tt := range tests {
		if result := BitwiseXor(tt.a, tt.b); result != tt.expected {
			t.Errorf("BitwiseXor(%d, %d) = %d; want %d", tt.a, tt.b, result, tt.expected)
		}
	}
}

func TestLeftShift(t *testing.T) {
	tests := []struct {
		a, shift int
		expected int
	}{
		{1, 3, 8},  // 1 << 3 = 8
		{5, 2, 20}, // 5 << 2 = 20
		{3, 1, 6},  // 3 << 1 = 6
	}

	for _, tt := range tests {
		if result := LeftShift(tt.a, tt.shift); result != tt.expected {
			t.Errorf("LeftShift(%d, %d) = %d; want %d", tt.a, tt.shift, result, tt.expected)
		}
	}
}

func TestRightShift(t *testing.T) {
	tests := []struct {
		a, shift int
		expected int
	}{
		{8, 3, 1},  // 8 >> 3 = 1
		{20, 2, 5}, // 20 >> 2 = 5
		{6, 1, 3},  // 6 >> 1 = 3
	}

	for _, tt := range tests {
		if result := RightShift(tt.a, tt.shift); result != tt.expected {
			t.Errorf("RightShift(%d, %d) = %d; want %d", tt.a, tt.shift, result, tt.expected)
		}
	}
}

func TestNegate(t *testing.T) {
	tests := []struct {
		value    int
		expected int
	}{
		{5, -5},
		{-5, 5},
		{0, 0},
		{100, -100},
	}

	for _, tt := range tests {
		if result := Negate(tt.value); result != tt.expected {
			t.Errorf("Negate(%d) = %d; want %d", tt.value, result, tt.expected)
		}
	}
}

func TestPositive(t *testing.T) {
	tests := []struct {
		value    int
		expected int
	}{
		{5, 5},
		{-5, -5}, // Unary + doesn't change sign
		{0, 0},
	}

	for _, tt := range tests {
		if result := Positive(tt.value); result != tt.expected {
			t.Errorf("Positive(%d) = %d; want %d", tt.value, result, tt.expected)
		}
	}
}

func TestNot(t *testing.T) {
	tests := []struct {
		flag     bool
		expected bool
	}{
		{true, false},
		{false, true},
	}

	for _, tt := range tests {
		if result := Not(tt.flag); result != tt.expected {
			t.Errorf("Not(%v) = %v; want %v", tt.flag, result, tt.expected)
		}
	}
}

func TestAddAssign(t *testing.T) {
	tests := []struct {
		value, delta int
		expected     int
	}{
		{10, 5, 15},
		{0, 10, 10},
		{-5, 3, -2},
	}

	for _, tt := range tests {
		if result := AddAssign(tt.value, tt.delta); result != tt.expected {
			t.Errorf("AddAssign(%d, %d) = %d; want %d", tt.value, tt.delta, result, tt.expected)
		}
	}
}

func TestSubtractAssign(t *testing.T) {
	tests := []struct {
		value, delta int
		expected     int
	}{
		{10, 5, 5},
		{0, 10, -10},
		{-5, 3, -8},
	}

	for _, tt := range tests {
		if result := SubtractAssign(tt.value, tt.delta); result != tt.expected {
			t.Errorf("SubtractAssign(%d, %d) = %d; want %d", tt.value, tt.delta, result, tt.expected)
		}
	}
}

func TestMultiplyAssign(t *testing.T) {
	tests := []struct {
		value, factor int
		expected      int
	}{
		{10, 5, 50},
		{3, 4, 12},
		{-2, 3, -6},
	}

	for _, tt := range tests {
		if result := MultiplyAssign(tt.value, tt.factor); result != tt.expected {
			t.Errorf("MultiplyAssign(%d, %d) = %d; want %d", tt.value, tt.factor, result, tt.expected)
		}
	}
}

func TestDivideAssign(t *testing.T) {
	tests := []struct {
		value, divisor int
		expected       int
	}{
		{20, 5, 4},
		{15, 3, 5},
		{-10, 2, -5},
	}

	for _, tt := range tests {
		if result := DivideAssign(tt.value, tt.divisor); result != tt.expected {
			t.Errorf("DivideAssign(%d, %d) = %d; want %d", tt.value, tt.divisor, result, tt.expected)
		}
	}
}
