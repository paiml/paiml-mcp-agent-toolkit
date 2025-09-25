// C# test file
using System;

class Program
{
    static void Main(string[] args)
    {
        Console.WriteLine("Hello C#");
    }

    static int AddNumbers(int a, int b)
    {
        return a + b;
    }

    static int Fibonacci(int n)
    {
        if (n <= 1)
            return n;
        return Fibonacci(n - 1) + Fibonacci(n - 2);
    }
}

class Calculator
{
    private double value;

    public Calculator()
    {
        this.value = 0.0;
    }

    public void Calculate(char op, double val)
    {
        switch (op)
        {
            case '+':
                this.value += val;
                break;
            case '-':
                this.value -= val;
                break;
        }
    }

    public double GetValue()
    {
        return this.value;
    }
}