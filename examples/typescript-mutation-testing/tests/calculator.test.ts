/**
 * Comprehensive tests for calculator module.
 *
 * These tests demonstrate mutation testing with PMAT.
 */

import { describe, test, expect } from '@jest/globals';
import {
  add,
  subtract,
  multiply,
  divide,
  isEven,
  max,
  factorial,
  isPrime,
} from '../src/calculator';

describe('Arithmetic Operations', () => {
  test('add should correctly add two numbers', () => {
    expect(add(2, 3)).toBe(5);
    expect(add(-1, 1)).toBe(0);
    expect(add(0, 0)).toBe(0);
  });

  test('subtract should correctly subtract two numbers', () => {
    expect(subtract(5, 3)).toBe(2);
    expect(subtract(1, 1)).toBe(0);
    expect(subtract(0, 5)).toBe(-5);
  });

  test('multiply should correctly multiply two numbers', () => {
    expect(multiply(3, 4)).toBe(12);
    expect(multiply(0, 5)).toBe(0);
    expect(multiply(-2, 3)).toBe(-6);
  });

  test('divide should correctly divide two numbers', () => {
    expect(divide(10, 2)).toBe(5);
    expect(divide(7, 2)).toBe(3.5);
    expect(divide(5, 0)).toBeNull();
  });
});

describe('Logical Operations', () => {
  test('isEven should detect even numbers', () => {
    expect(isEven(4)).toBe(true);
    expect(isEven(0)).toBe(true);
    expect(isEven(3)).toBe(false);
    expect(isEven(-1)).toBe(false);
  });

  test('max should return maximum value', () => {
    expect(max(5, 3)).toBe(5);
    expect(max(2, 8)).toBe(8);
    expect(max(4, 4)).toBe(4);
  });
});

describe('Complex Operations', () => {
  test('factorial should calculate factorial correctly', () => {
    expect(factorial(0)).toBe(1);
    expect(factorial(1)).toBe(1);
    expect(factorial(5)).toBe(120);
    expect(factorial(10)).toBe(3628800);
  });

  test('factorial should throw error for negative numbers', () => {
    expect(() => factorial(-1)).toThrow('Factorial is not defined for negative numbers');
  });

  test('isPrime should detect prime numbers', () => {
    expect(isPrime(0)).toBe(false);
    expect(isPrime(1)).toBe(false);
    expect(isPrime(2)).toBe(true);
    expect(isPrime(3)).toBe(true);
    expect(isPrime(4)).toBe(false);
    expect(isPrime(5)).toBe(true);
    expect(isPrime(9)).toBe(false);
    expect(isPrime(11)).toBe(true);
    expect(isPrime(15)).toBe(false);
    expect(isPrime(17)).toBe(true);
  });
});

describe('Edge Cases', () => {
  test('add with large numbers', () => {
    expect(add(1000000, 2000000)).toBe(3000000);
  });

  test('multiply by zero', () => {
    expect(multiply(999, 0)).toBe(0);
    expect(multiply(0, 999)).toBe(0);
  });

  test('divide by zero returns null', () => {
    expect(divide(10, 0)).toBeNull();
    expect(divide(0, 0)).toBeNull();
  });

  test('isEven with negative numbers', () => {
    expect(isEven(-2)).toBe(true);
    expect(isEven(-4)).toBe(true);
    expect(isEven(-3)).toBe(false);
  });

  test('max with equal numbers', () => {
    expect(max(0, 0)).toBe(0);
    expect(max(-5, -5)).toBe(-5);
    expect(max(100, 100)).toBe(100);
  });

  test('factorial of zero', () => {
    expect(factorial(0)).toBe(1);
  });

  test('isPrime for 2', () => {
    expect(isPrime(2)).toBe(true);
  });

  test('isPrime for large numbers', () => {
    expect(isPrime(97)).toBe(true);
    expect(isPrime(100)).toBe(false);
  });
});
