// Go test file
package main

import "fmt"

func main() {
	fmt.Println("Hello Go")
}

func addNumbers(a, b int) int {
	return a + b
}

type Calculator struct {
	value float64
}

func NewCalculator() *Calculator {
	return &Calculator{value: 0.0}
}

func (c *Calculator) Calculate(op rune, val float64) {
	switch op {
	case '+':
		c.value += val
	case '-':
		c.value -= val
	}
}

func fibonacci(n int) int {
	if n <= 1 {
		return n
	}
	return fibonacci(n-1) + fibonacci(n-2)
}