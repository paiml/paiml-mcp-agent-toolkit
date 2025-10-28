"""Comprehensive tests for calculator module.

These tests demonstrate mutation testing with PMAT.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

import pytest
from calculator import (
    add, subtract, multiply, divide, is_even, max_value, factorial, is_prime
)


class TestArithmeticOperations:
    """Test basic arithmetic operations."""

    def test_add(self):
        """Test addition."""
        assert add(2, 3) == 5
        assert add(-1, 1) == 0
        assert add(0, 0) == 0

    def test_subtract(self):
        """Test subtraction."""
        assert subtract(5, 3) == 2
        assert subtract(1, 1) == 0
        assert subtract(0, 5) == -5

    def test_multiply(self):
        """Test multiplication."""
        assert multiply(3, 4) == 12
        assert multiply(0, 5) == 0
        assert multiply(-2, 3) == -6

    def test_divide(self):
        """Test division."""
        assert divide(10, 2) == 5.0
        assert divide(7, 2) == 3.5
        assert divide(5, 0) is None


class TestLogicalOperations:
    """Test logical operations."""

    def test_is_even(self):
        """Test even number detection."""
        assert is_even(4)
        assert is_even(0)
        assert not is_even(3)
        assert not is_even(-1)

    def test_max_value(self):
        """Test maximum value calculation."""
        assert max_value(5, 3) == 5
        assert max_value(2, 8) == 8
        assert max_value(4, 4) == 4


class TestComplexOperations:
    """Test more complex operations."""

    def test_factorial(self):
        """Test factorial calculation."""
        assert factorial(0) == 1
        assert factorial(1) == 1
        assert factorial(5) == 120
        assert factorial(10) == 3628800

    def test_factorial_negative(self):
        """Test factorial with negative input."""
        with pytest.raises(ValueError):
            factorial(-1)

    def test_is_prime(self):
        """Test prime number detection."""
        assert not is_prime(0)
        assert not is_prime(1)
        assert is_prime(2)
        assert is_prime(3)
        assert not is_prime(4)
        assert is_prime(5)
        assert not is_prime(9)
        assert is_prime(11)
        assert not is_prime(15)
        assert is_prime(17)


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_add_large_numbers(self):
        """Test addition with large numbers."""
        assert add(1000000, 2000000) == 3000000

    def test_multiply_by_zero(self):
        """Test multiplication by zero."""
        assert multiply(999, 0) == 0
        assert multiply(0, 999) == 0

    def test_divide_by_zero(self):
        """Test division by zero returns None."""
        assert divide(10, 0) is None
        assert divide(0, 0) is None

    def test_is_even_negative(self):
        """Test is_even with negative numbers."""
        assert is_even(-2)
        assert is_even(-4)
        assert not is_even(-3)

    def test_max_value_equal(self):
        """Test max_value with equal numbers."""
        assert max_value(0, 0) == 0
        assert max_value(-5, -5) == -5
        assert max_value(100, 100) == 100

    def test_factorial_zero(self):
        """Test factorial of zero."""
        assert factorial(0) == 1

    def test_is_prime_two(self):
        """Test that 2 is prime."""
        assert is_prime(2)

    def test_is_prime_large(self):
        """Test prime detection with larger numbers."""
        assert is_prime(97)
        assert not is_prime(100)
