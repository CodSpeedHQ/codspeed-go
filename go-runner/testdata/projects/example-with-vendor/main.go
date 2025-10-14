package examplewithvendor

// LocalFib is a local implementation of Fibonacci
func LocalFib(n int) int {
	if n <= 1 {
		return n
	}
	return LocalFib(n-1) + LocalFib(n-2)
}

// Add is a simple addition function to use with testify assertions
func Add(a, b int) int {
	return a + b
}
