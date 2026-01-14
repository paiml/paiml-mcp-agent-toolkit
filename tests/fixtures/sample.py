# Simple Python fixture for DAP integration testing
def calculate_sum(x, y):
    result = x + y
    doubled = result * 2
    print(f"Result: {doubled}")
    return doubled

def main():
    a = 10
    b = 20
    sum_value = calculate_sum(a, b)
    print(f"Sum: {sum_value}")

if __name__ == "__main__":
    main()
