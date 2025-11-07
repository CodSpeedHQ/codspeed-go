package subpackages

import (
	"bytes"
	"io"
	"testing"
	"testing/fstest"
	"testing/iotest"
	"testing/synctest"
)

// BenchmarkBufferWrite benchmarks writing to a buffer using io.Writer interface
func BenchmarkBufferWrite(b *testing.B) {
	buf := &Buffer{}
	data := []byte("hello world hello world hello world")

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		buf.Write(data)
	}
}

// BenchmarkBufferRead benchmarks reading from a buffer
func BenchmarkBufferRead(b *testing.B) {
	buf := &Buffer{}
	buf.Write([]byte("hello world hello world hello world"))
	readBuf := make([]byte, 10)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		buf.Read(readBuf)
	}
}

// BenchmarkIOTestReader benchmarks using iotest package
func BenchmarkIOTestReader(b *testing.B) {
	data := []byte("hello world hello world hello world")
	reader := bytes.NewReader(data)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		// Create a reader that returns 1 byte at a time
		oneByteReader := iotest.OneByteReader(bytes.NewReader(data))
		io.Copy(io.Discard, oneByteReader)
		reader.Reset(data)
	}
}

// BenchmarkFSTestFS benchmarks the fstest package
func BenchmarkFSTestFS(b *testing.B) {
	fs := fstest.MapFS{
		"hello.txt": {Data: []byte("hello world")},
		"test.txt":  {Data: []byte("test data")},
	}

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		data, _ := fs.ReadFile("hello.txt")
		_ = data
	}
}

// TestSyncTestConcurrency tests using synctest for concurrent operations
func TestSyncTestConcurrency(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		counter := &Counter{}
		done := make(chan bool, 1)

		// Simulate concurrent increments with goroutine
		go func() {
			for i := 0; i < 100; i++ {
				counter.Increment()
			}
			done <- true
		}()

		// Wait for goroutine to complete
		<-done

		// Verify final value
		finalVal := counter.Value()
		if finalVal != 100 {
			t.Errorf("expected counter to be 100, got %d", finalVal)
		}
	})
}

// BenchmarkCounterWithMutex benchmarks counter increments with mutex protection
func BenchmarkCounterWithMutex(b *testing.B) {
	counter := &Counter{}

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		counter.Increment()
	}
}

// BenchmarkWriterFunc benchmarks using a function as a writer
func BenchmarkWriterFunc(b *testing.B) {
	writeCount := 0
	fn := WriterFunc(func(p []byte) (n int, err error) {
		writeCount++
		return len(p), nil
	})

	data := []byte("hello world")

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		fn.Write(data)
	}
}
