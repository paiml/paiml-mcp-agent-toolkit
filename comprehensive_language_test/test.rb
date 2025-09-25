# Simple Ruby calculator module
module Calculator
  # Add two numbers
  def self.add(a, b)
    a + b
  end

  # Multiply two numbers
  def self.multiply(a, b)
    a * b
  end

  # Calculate factorial using recursion
  def self.factorial(n)
    return 1 if n <= 1
    n * factorial(n - 1)
  end
end

# Math utility class
class MathUtils
  # Check if number is prime
  def is_prime(num)
    return false if num < 2
    (2..Math.sqrt(num)).each do |i|
      return false if num % i == 0
    end
    true
  end

  # Generate Fibonacci sequence
  def fibonacci(n)
    return [] if n <= 0
    return [0] if n == 1
    return [0, 1] if n == 2

    fib = [0, 1]
    (2...n).each do |i|
      fib[i] = fib[i-1] + fib[i-2]
    end
    fib
  end
end

# Usage example
calc = Calculator
utils = MathUtils.new

puts calc.add(5, 3)
puts calc.factorial(5)
puts utils.is_prime(17)
puts utils.fibonacci(10)