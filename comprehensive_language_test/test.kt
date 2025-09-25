// Kotlin test file
fun main() {
    println("Hello Kotlin")
}

fun addNumbers(a: Int, b: Int): Int {
    return a + b
}

fun fibonacci(n: Int): Int {
    if (n <= 1) return n
    return fibonacci(n - 1) + fibonacci(n - 2)
}

class Calculator {
    private var value: Double = 0.0

    fun calculate(op: Char, `val`: Double) {
        when (op) {
            '+' -> value += `val`
            '-' -> value -= `val`
        }
    }

    fun getValue(): Double {
        return value
    }
}

data class Point(val x: Int, val y: Int)

object MathUtils {
    fun isPrime(n: Int): Boolean {
        if (n <= 1) return false
        for (i in 2 until n) {
            if (n % i == 0) return false
        }
        return true
    }
}