"""Simple calculator module for demonstrating mutation testing.

This example shows how mutation testing can detect gaps in test coverage.
"""

from typing import Optional


def add(a: int, b: int) -> int:
    """Add two numbers."""
    return a + b


def subtract(a: int, b: int) -> int:
    """Subtract two numbers."""
    return a - b


def multiply(a: int, b: int) -> int:
    """Multiply two numbers."""
    return a * b


def divide(a: int, b: int) -> Optional[float]:
    """Divide two numbers (returns None if division by zero)."""
    if b == 0:
        return None
    return a / b


def is_even(n: int) -> bool:
    """Check if a number is even."""
    return n % 2 == 0


def max_value(a: int, b: int) -> int:
    """Calculate the maximum of two numbers."""
    if a > b:
        return a
    else:
        return b


def factorial(n: int) -> int:
    """Calculate factorial (iterative)."""
    if n < 0:
        raise ValueError("Factorial is not defined for negative numbers")
    if n == 0:
        return 1

    result = 1
    for i in range(1, n + 1):
        result *= i
    return result


def is_prime(n: int) -> bool:
    """Check if a number is prime."""
    if n <= 1:
        return False
    if n == 2:
        return True
    if n % 2 == 0:
        return False

    limit = int(n ** 0.5)
    for i in range(3, limit + 1, 2):
        if n % i == 0:
            return False
    return True
