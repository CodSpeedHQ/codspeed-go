package fib

// Fibonacci calculates the nth Fibonacci number recursively
func Fibonacci(n int) int {
	if n <= 1 {
		return n
	}
	return Fibonacci(n-1) + Fibonacci(n-2)
}

func privateFibonacci(n int) int {
	if n <= 1 {
		return n
	}
	return privateFibonacci(n-1) + privateFibonacci(n-2)
}
