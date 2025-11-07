package subpackages

import (
	"bytes"
	"sync"
)

// Buffer is a simple wrapper around bytes.Buffer for demonstration
type Buffer struct {
	b bytes.Buffer
}

// Write writes data to the buffer
func (b *Buffer) Write(p []byte) (n int, err error) {
	return b.b.Write(p)
}

// Read reads data from the buffer
func (b *Buffer) Read(p []byte) (n int, err error) {
	return b.b.Read(p)
}

// String returns the buffer contents as a string
func (b *Buffer) String() string {
	return b.b.String()
}

// Counter is a mutex-protected counter for synctest demonstration
type Counter struct {
	mu    sync.Mutex
	value int
}

// Increment increments the counter
func (c *Counter) Increment() {
	c.mu.Lock()
	c.value++
	c.mu.Unlock()
}

// Value returns the current counter value
func (c *Counter) Value() int {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.value
}

// WriterFunc is a function type that implements io.Writer
type WriterFunc func(p []byte) (n int, err error)

// Write implements io.Writer
func (f WriterFunc) Write(p []byte) (n int, err error) {
	return f(p)
}
