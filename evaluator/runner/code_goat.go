package runner

/*
#include "code_goat.h"
#include <stdlib.h> // C.String

#cgo LDFLAGS: -L/usr/local/lib -ljudger
*/
import "C"

import (
	"fmt"
	"time"
	"unsafe"

	"github.com/aintbe/code-goat/evaluator/config"
)

func RunCodeGoat(spec *config.JudgeSpec) (*JudgeResult, error) {
	startTime := time.Now()

	// Trace allocated memory inside C heap and free all of them before this
	// function returns. This prevents memory leak after running.
	var cHeapToFree []*C.char
	defer func() {
		for _, cHeap := range cHeapToFree {
			if cHeap != nil {
				C.free(unsafe.Pointer(cHeap))
			}
		}
		fmt.Println("+ Released all c pointers.")
	}()

	// Allocate memory into C heap and return a pointer to it.
	allocate := func(goStr string) *C.char {
		if goStr == "" {
			return nil
		}
		cStr := C.CString(goStr)
		cHeapToFree = append(cHeapToFree, cStr)
		return cStr
	}

	// Convert Go Spec into C Spec to pass to Rust FFI function.
	cSpec := C.CJudgeSpec{
		exe_path:    allocate(spec.ExePath),
		input_path:  allocate(spec.InputPath),
		output_path: allocate(spec.OutputPath),
		error_path:  allocate(spec.ErrorPath),
		answer_path: allocate(spec.AnswerPath),
		args:        allocate(""),
		envs:        allocate(""),
		resource_limit: C.CResourceLimit{
			memory:    C.uint32_t(spec.ResourceLimit.Memory),
			cpu_time:  C.uint64_t(spec.ResourceLimit.CpuTime),
			real_time: C.uint32_t(spec.ResourceLimit.RealTime),
			stack:     C.uint64_t(spec.ResourceLimit.Stack),
			n_process: C.uint64_t(spec.ResourceLimit.NProcess),
			output:    C.uint64_t(spec.ResourceLimit.Output),
		},
	}

	// Call Rust FFI function to run judger.
	res := C.judger_judge(cSpec)

	// Copy returned result into Go heap and free Rust heap using provided function.
	defer C.judger_free(res)

	return NewJudgeResult(C.GoString(res), CodeGoat, startTime)
}
