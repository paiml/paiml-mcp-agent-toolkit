/**
 * Simple calculator module for demonstrating mutation testing.
 *
 * This example shows how mutation testing can detect gaps in test coverage.
 */

/**
 * Add two numbers.
 */
export function add(a: number, b: number): number {
  return a + b;
}

/**
 * Subtract two numbers.
 */
export function subtract(a: number, b: number): number {
  return a - b;
}

/**
 * Multiply two numbers.
 */
export function multiply(a: number, b: number): number {
  return a * b;
}

/**
 * Divide two numbers (returns null if division by zero).
 */
export function divide(a: number, b: number): number | null {
  if (b === 0) {
    return null;
  }
  return a / b;
}

/**
 * Check if a number is even.
 */
export function isEven(n: number): boolean {
  return n % 2 === 0;
}

/**
 * Calculate the maximum of two numbers.
 */
export function max(a: number, b: number): number {
  if (a > b) {
    return a;
  } else {
    return b;
  }
}

/**
 * Calculate factorial (iterative).
 */
export function factorial(n: number): number {
  if (n < 0) {
    throw new Error('Factorial is not defined for negative numbers');
  }
  if (n === 0) {
    return 1;
  }

  let result = 1;
  for (let i = 1; i <= n; i++) {
    result *= i;
  }
  return result;
}

/**
 * Check if a number is prime.
 */
export function isPrime(n: number): boolean {
  if (n <= 1) {
    return false;
  }
  if (n === 2) {
    return true;
  }
  if (n % 2 === 0) {
    return false;
  }

  const limit = Math.floor(Math.sqrt(n));
  for (let i = 3; i <= limit; i += 2) {
    if (n % i === 0) {
      return false;
    }
  }
  return true;
}
