package testing

/*
#cgo CFLAGS: -I@@INSTRUMENT_HOOKS_DIR@@/includes -Wno-format -Wno-format-security
#include "@@INSTRUMENT_HOOKS_DIR@@/dist/core.c"
*/
import "C"
import (
	"os"
	"runtime"
	"unsafe"
)

var integrationVersion = "@@GO_RUNNER_VERSION@@"

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

	C.instrument_hooks_set_integration(i.hooks, nameC, versionC)
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

func (i *InstrumentHooks) SetExecutedBenchmark(pid int32, name string) {
	if i.hooks == nil {
		return
	}
	nameC := C.CString(name)
	defer C.free(unsafe.Pointer(nameC))

	C.instrument_hooks_set_executed_benchmark(i.hooks, C.int32_t(pid), nameC)
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
	pid := C.int32_t(os.Getpid())
	C.instrument_hooks_add_marker(i.hooks, C.int32_t(pid), C.MARKER_TYPE_BENCHMARK_START, C.uint64_t(startTimestamp))
	C.instrument_hooks_add_marker(i.hooks, C.int32_t(pid), C.MARKER_TYPE_BENCHMARK_END, C.uint64_t(endTimestamp))
}

func (i *InstrumentHooks) SetEnvironment(sectionName, key, value string) {
	if i.hooks == nil {
		return
	}
	sectionNameC := C.CString(sectionName)
	keyC := C.CString(key)
	valueC := C.CString(value)
	defer C.free(unsafe.Pointer(sectionNameC))
	defer C.free(unsafe.Pointer(keyC))
	defer C.free(unsafe.Pointer(valueC))

	C.instrument_hooks_set_environment(i.hooks, sectionNameC, keyC, valueC)
}

func (i *InstrumentHooks) WriteEnvironment(pid int32) {
	if i.hooks == nil {
		return
	}
	C.instrument_hooks_write_environment(i.hooks, C.int32_t(pid))
}
