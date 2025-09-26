using System;

namespace TestApp
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("Test C# file");
        }

        static int Calculate(int x)
        {
            if (x > 10)
            {
                return x * 2;
            }
            return x + 5;
        }
    }
}