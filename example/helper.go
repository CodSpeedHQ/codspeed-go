package example

import "time"

//go:noinline
func recursiveFib(n int) uint64 {
	if n <= 1 {
		return uint64(n)
	}
	return recursiveFib(n-1) + recursiveFib(n-2)
}

//go:noinline
func expensiveOperation() uint64 {
	// Large memory allocation
	data := make([]uint64, 1024*1024) // 8 MiB allocation
	for i := range data {
		data[i] = 42
	}

	// Expensive recursive computation that will dominate flamegraph
	fibResult := recursiveFib(30)

	// More expensive work - sum the data
	sum := uint64(0)
	for _, v := range data {
		sum += v
	}

	return sum + fibResult
}

//go:noinline
func doWork(n int) uint64 {
	if n <= 1 {
		return uint64(n)
	}
	return doWork(n-1) + doWork(n-2)
}

func actualWork() uint64 {
	time.Sleep(1 * time.Millisecond)
	result := doWork(30)
	return 42 + result
}
