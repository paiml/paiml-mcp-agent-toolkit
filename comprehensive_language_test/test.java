// Java test file
public class Test {
    public static void main(String[] args) {
        System.out.println("Hello Java");
    }

    public static int addNumbers(int a, int b) {
        return a + b;
    }

    public static int fibonacci(int n) {
        if (n <= 1) {
            return n;
        }
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

class Calculator {
    private double value;

    public Calculator() {
        this.value = 0.0;
    }

    public void calculate(char op, double val) {
        switch (op) {
            case '+':
                this.value += val;
                break;
            case '-':
                this.value -= val;
                break;
        }
    }

    public double getValue() {
        return this.value;
    }
}