package fib

// Fibonacci calculates the nth Fibonacci number recursively
func Fibonacci(n int) int {
	if n <= 1 {
		return n
	}
	return Fibonacci(n-1) + Fibonacci(n-2)
}
