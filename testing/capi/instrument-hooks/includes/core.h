// This file was manually created and exposes the functions of this library.
// TODO: Can we automatically generate this file?

#ifndef CORE_H
#define CORE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifndef _WIN32
#include "callgrind.h"
#include "valgrind.h"
#else
#define CALLGRIND_START_INSTRUMENTATION
#define CALLGRIND_STOP_INSTRUMENTATION
#define CALLGRIND_ZERO_STATS
#endif

typedef uint64_t *InstrumentHooks;

InstrumentHooks *instrument_hooks_init(void);
void instrument_hooks_deinit(InstrumentHooks *);

bool instrument_hooks_is_instrumented(InstrumentHooks *);
int8_t instrument_hooks_start_benchmark(InstrumentHooks *);
int8_t instrument_hooks_stop_benchmark(InstrumentHooks *);
int8_t instrument_hooks_set_executed_benchmark(InstrumentHooks *, int32_t pid,
                                               const char *uri);
// Deprecated: use instrument_hooks_set_executed_benchmark instead
int8_t instrument_hooks_executed_benchmark(InstrumentHooks *, int32_t pid,
                                           const char *uri);
int8_t instrument_hooks_set_integration(InstrumentHooks *, const char *name,
                                        const char *version);

int8_t callgrind_start_instrumentation();
int8_t callgrind_stop_instrumentation();

// Feature flags for instrument hooks

typedef enum {
  FEATURE_DISABLE_CALLGRIND_MARKERS = 0,
} instrument_hooks_feature_t;

void instrument_hooks_set_feature(instrument_hooks_feature_t feature,
                                  bool enabled);

// Header functions that will be inlined. This can be used by languages that
// directly consume the headers such as C or C++. This will allow for more
// precise tracking of the benchmark performance.

static inline int8_t instrument_hooks_start_benchmark_inline(
    InstrumentHooks *instance) {
  instrument_hooks_set_feature(FEATURE_DISABLE_CALLGRIND_MARKERS, true);
  if (instrument_hooks_start_benchmark(instance) != 0) {
    return 1;
  }

  CALLGRIND_ZERO_STATS;
  CALLGRIND_START_INSTRUMENTATION;
  return 0;
}

static inline int8_t instrument_hooks_stop_benchmark_inline(
    InstrumentHooks *instance) {
  CALLGRIND_STOP_INSTRUMENTATION;
  return instrument_hooks_stop_benchmark(instance);
}

#ifdef __cplusplus
}
#endif

#endif
