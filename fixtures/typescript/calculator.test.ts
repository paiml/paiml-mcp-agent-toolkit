import { describe, test, expect } from 'vitest';
import {
    add,
    subtract,
    multiply,
    divide,
    isPositive,
    isNegative,
    isZero,
    isEqual,
    isNotEqual,
    max,
    min,
    getNestedValue,
    getValueOrDefault,
    fetchValue,
    double,
    negate,
    isString,
} from './calculator';

describe('Basic arithmetic', () => {
    test('add returns sum', () => {
        expect(add(2, 3)).toBe(5);
        expect(add(-1, 1)).toBe(0);
        expect(add(0, 0)).toBe(0);
    });

    test('subtract returns difference', () => {
        expect(subtract(5, 3)).toBe(2);
        expect(subtract(1, 1)).toBe(0);
        expect(subtract(0, 5)).toBe(-5);
    });

    test('multiply returns product', () => {
        expect(multiply(2, 3)).toBe(6);
        expect(multiply(0, 100)).toBe(0);
        expect(multiply(-2, 3)).toBe(-6);
    });

    test('divide returns quotient', () => {
        expect(divide(6, 2)).toBe(3);
        expect(divide(5, 2)).toBe(2.5);
        expect(() => divide(5, 0)).toThrow('Division by zero');
    });
});

describe('Comparison operators', () => {
    test('isPositive checks if number is positive', () => {
        expect(isPositive(5)).toBe(true);
        expect(isPositive(0)).toBe(false);
        expect(isPositive(-5)).toBe(false);
    });

    test('isNegative checks if number is negative', () => {
        expect(isNegative(-5)).toBe(true);
        expect(isNegative(0)).toBe(false);
        expect(isNegative(5)).toBe(false);
    });

    test('isZero checks if number is zero', () => {
        expect(isZero(0)).toBe(true);
        expect(isZero(1)).toBe(false);
        expect(isZero(-1)).toBe(false);
    });

    test('isEqual checks equality', () => {
        expect(isEqual(5, 5)).toBe(true);
        expect(isEqual(5, 6)).toBe(false);
    });

    test('isNotEqual checks inequality', () => {
        expect(isNotEqual(5, 6)).toBe(true);
        expect(isNotEqual(5, 5)).toBe(false);
    });
});

describe('Min/max functions', () => {
    test('max returns maximum', () => {
        expect(max(5, 3)).toBe(5);
        expect(max(3, 5)).toBe(5);
        expect(max(5, 5)).toBe(5);
    });

    test('min returns minimum', () => {
        expect(min(5, 3)).toBe(3);
        expect(min(3, 5)).toBe(3);
        expect(min(5, 5)).toBe(5);
    });
});

describe('TypeScript-specific features', () => {
    test('optional chaining', () => {
        expect(getNestedValue({ nested: { value: 42 } })).toBe(42);
        expect(getNestedValue({ nested: {} })).toBeUndefined();
        expect(getNestedValue({})).toBeUndefined();
        expect(getNestedValue(undefined)).toBeUndefined();
    });

    test('nullish coalescing', () => {
        expect(getValueOrDefault(10, 0)).toBe(10);
        expect(getValueOrDefault(null, 5)).toBe(5);
        expect(getValueOrDefault(undefined, 5)).toBe(5);
        expect(getValueOrDefault(0, 5)).toBe(0); // 0 is not nullish!
    });

    test('async/await', async () => {
        const result = await fetchValue();
        expect(result).toBe(42);
    });

    test('arrow functions', () => {
        expect(double(5)).toBe(10);
        expect(negate(5)).toBe(-5);
        expect(negate(-5)).toBe(5);
    });

    test('type guards', () => {
        expect(isString('hello')).toBe(true);
        expect(isString(42)).toBe(false);
        expect(isString(null)).toBe(false);
    });
});
