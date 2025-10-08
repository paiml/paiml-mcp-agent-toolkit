"""Pytest tests for calculator."""
import pytest
from calculator import (
    add, subtract, multiply, divide, power,
    is_positive, is_even, logical_and, logical_or,
    is_none, contains
)

def test_add():
    assert add(2, 3) == 5
    assert add(-1, 1) == 0
    assert add(0, 0) == 0

def test_subtract():
    assert subtract(5, 3) == 2
    assert subtract(0, 5) == -5

def test_multiply():
    assert multiply(3, 4) == 12
    assert multiply(0, 5) == 0

def test_divide():
    assert divide(10, 2) == 5.0
    with pytest.raises(ValueError):
        divide(10, 0)

def test_power():
    assert power(2, 3) == 8
    assert power(5, 0) == 1

def test_is_positive():
    assert is_positive(5) == True
    assert is_positive(-5) == False
    assert is_positive(0) == False

def test_is_even():
    assert is_even(4) == True
    assert is_even(5) == False

def test_logical_and():
    assert logical_and(True, True) == True
    assert logical_and(True, False) == False
    assert logical_and(False, False) == False

def test_logical_or():
    assert logical_or(True, False) == True
    assert logical_or(False, False) == False

def test_is_none():
    assert is_none(None) == True
    assert is_none(0) == False
    assert is_none("") == False

def test_contains():
    assert contains(1, [1, 2, 3]) == True
    assert contains(4, [1, 2, 3]) == False
