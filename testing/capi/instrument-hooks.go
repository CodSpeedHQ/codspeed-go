package capi

/*
#cgo CFLAGS: -I${SRCDIR}/instrument-hooks/includes -Wno-format -Wno-format-security
#include "instrument-hooks/dist/core.c"

#define MARKER_TYPE_BENCHMARK_START c_MARKER_TYPE_BENCHMARK_START__249
#define MARKER_TYPE_BENCHMARK_END c_MARKER_TYPE_BENCHMARK_END__250
typedef struct instruments_root_InstrumentHooks__547 InstrumentHooks;
*/
import "C"
import (
	"os"
	"runtime"
	"unsafe"
)

// This will be set in the go-runner
var integrationVersion = "dev"

type InstrumentHooks struct {
	hooks *C.InstrumentHooks
}

func NewInstrumentHooks() *InstrumentHooks {
	inst := &InstrumentHooks{
		hooks: C.instrument_hooks_init(),
	}
	runtime.SetFinalizer(inst, (*InstrumentHooks).cleanup)
	inst.SetIntegration("codspeed-go", integrationVersion)
	return inst
}

func (i *InstrumentHooks) Close() {
	if i.hooks != nil {
		C.instrument_hooks_deinit(i.hooks)
		i.hooks = nil
		runtime.SetFinalizer(i, nil)
	}
}

func (i *InstrumentHooks) cleanup() {
	i.Close()
}

func (i *InstrumentHooks) SetIntegration(name, version string) {
	if i.hooks == nil {
		return
	}
	nameC := C.CString(name)
	versionC := C.CString(version)
	defer C.free(unsafe.Pointer(nameC))
	defer C.free(unsafe.Pointer(versionC))

	C.instrument_hooks_set_integration(i.hooks, (*C.uint8_t)(unsafe.Pointer(nameC)), (*C.uint8_t)(unsafe.Pointer(versionC)))
}

func (i *InstrumentHooks) StartBenchmark() {
	if i.hooks != nil {
		C.instrument_hooks_start_benchmark(i.hooks)
	}
}

func (i *InstrumentHooks) StopBenchmark() {
	if i.hooks != nil {
		C.instrument_hooks_stop_benchmark(i.hooks)
	}
}

func (i *InstrumentHooks) SetExecutedBenchmark(pid uint32, name string) {
	if i.hooks == nil {
		return
	}
	nameC := C.CString(name)
	defer C.free(unsafe.Pointer(nameC))

	C.instrument_hooks_set_executed_benchmark(i.hooks, C.uint(pid), (*C.uint8_t)(unsafe.Pointer(nameC)))
}

func (i *InstrumentHooks) IsInstrumented() bool {
	if i.hooks == nil {
		return false
	}
	return bool(C.instrument_hooks_is_instrumented(i.hooks))
}

func CurrentTimestamp() uint64 {
	return uint64(C.instrument_hooks_current_timestamp())
}

func (i *InstrumentHooks) AddBenchmarkTimestamps(startTimestamp, endTimestamp uint64) {
	if i.hooks == nil {
		return
	}
	pid := uint32(os.Getpid())
	C.instrument_hooks_add_marker(i.hooks, C.uint32_t(pid), C.MARKER_TYPE_BENCHMARK_START, C.uint64_t(startTimestamp))
	C.instrument_hooks_add_marker(i.hooks, C.uint32_t(pid), C.MARKER_TYPE_BENCHMARK_END, C.uint64_t(endTimestamp))
}
