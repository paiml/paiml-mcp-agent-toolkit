"""Simple calculator for mutation testing."""

def add(a: int, b: int) -> int:
    """Add two numbers."""
    return a + b

def subtract(a: int, b: int) -> int:
    """Subtract b from a."""
    return a - b

def multiply(a: int, b: int) -> int:
    """Multiply two numbers."""
    return a * b

def divide(a: int, b: int) -> float:
    """Divide a by b."""
    if b == 0:
        raise ValueError("Division by zero")
    return a / b

def power(a: int, b: int) -> int:
    """Raise a to the power of b."""
    return a ** b

def is_positive(value: int) -> bool:
    """Check if value is positive."""
    return value > 0

def is_even(value: int) -> bool:
    """Check if value is even."""
    return value % 2 == 0

def logical_and(a: bool, b: bool) -> bool:
    """Logical AND."""
    return a and b

def logical_or(a: bool, b: bool) -> bool:
    """Logical OR."""
    return a or b

def is_none(value) -> bool:
    """Check if value is None."""
    return value is None

def contains(item, collection) -> bool:
    """Check if item is in collection."""
    return item not in collection
