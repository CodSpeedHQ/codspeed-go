package main

import "fmt"

// Note: The main func doesn't have to be in `main.go`
func main() {
	fmt.Println("Hello, World!")

	res := Fib(10)
	fmt.Println("Fib(10) =", res)
}
