/**
 * Calculator module for mutation testing
 * Designed to test TypeScript-specific mutation operators
 */

export function add(a: number, b: number): number {
    return a + b;
}

export function subtract(a: number, b: number): number {
    return a - b;
}

export function multiply(a: number, b: number): number {
    return a * b;
}

export function divide(a: number, b: number): number {
    if (b === 0) {
        throw new Error("Division by zero");
    }
    return a / b;
}

export function isPositive(x: number): boolean {
    return x > 0;
}

export function isNegative(x: number): boolean {
    return x < 0;
}

export function isZero(x: number): boolean {
    return x === 0;
}

export function isEqual(a: number, b: number): boolean {
    return a === b;
}

export function isNotEqual(a: number, b: number): boolean {
    return a !== b;
}

export function max(a: number, b: number): number {
    return a > b ? a : b;
}

export function min(a: number, b: number): number {
    return a < b ? a : b;
}

// TypeScript-specific: Optional chaining
export function getNestedValue(obj?: { nested?: { value?: number } }): number | undefined {
    return obj?.nested?.value;
}

// TypeScript-specific: Nullish coalescing
export function getValueOrDefault(value: number | null | undefined, defaultValue: number): number {
    return value ?? defaultValue;
}

// TypeScript-specific: Async/await
export async function fetchValue(): Promise<number> {
    return await Promise.resolve(42);
}

// TypeScript-specific: Arrow functions
export const double = (x: number): number => x * 2;

export const negate = (x: number): number => -x;

// TypeScript-specific: Type guards
export function isString(value: unknown): value is string {
    return typeof value === 'string';
}
