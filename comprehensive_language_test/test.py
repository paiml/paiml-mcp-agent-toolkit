# Python test file
def main():
    print("Hello Python")

def add_numbers(a, b):
    return a + b

class Calculator:
    def __init__(self):
        self.value = 0.0

    def calculate(self, op, val):
        if op == '+':
            self.value += val
        elif op == '-':
            self.value -= val

def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

if __name__ == "__main__":
    main()