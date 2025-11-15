package adapter

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

	"github.com/aintbe/code-goat/evaluator/profile"
	"github.com/aintbe/code-goat/evaluator/utils"
)

func RunCodeGoat(spec *profile.JudgeSpec) (*JudgeResult, error) {
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
		exe_path:   allocate(spec.ExePath),
		input_path: allocate(spec.InputPath),
		// Do not grade inside judger to run it under the same condition.
		answer_path: allocate(""),
		output_path: allocate(spec.OutputPath),
		error_path:  allocate(spec.ErrorPath),
		args:        allocate(spec.Args.String()),
		envs:        allocate(spec.Envs.String()),
		scmp_policy: C.uint8_t(spec.ScmpPolicy),
		resource_limit: C.CResourceLimit{
			memory:    C.uint64_t(spec.ResourceLimit.Memory),
			cpu_time:  C.uint32_t(spec.ResourceLimit.CpuTime),
			real_time: C.uint32_t(spec.ResourceLimit.RealTime),
			stack:     C.uint32_t(spec.ResourceLimit.Stack),
			n_process: C.uint16_t(spec.ResourceLimit.NProcess),
			output:    C.uint32_t(spec.ResourceLimit.Output),
		},
	}

	// Configure logger inside judger. Uncomment below line to log into a files.
	// C.judger_configure_logger(allocate(spec.LogPath))
	C.judger_configure_logger(nil)

	// Call Rust FFI function to run judger.
	res := C.judger_judge(cSpec)
	elapsedTime := time.Since(startTime)

	// Copy returned result into Go heap and free Rust heap using provided function.
	defer C.judger_free(res)

	judgeResult, err := utils.Unmarshal[JudgeResult]([]byte(C.GoString(res)))
	if err != nil {
		return judgeResult, err
	}
	judgeResult.Judger = CodeGoat
	judgeResult.JudgeTime = elapsedTime

	err = judgeResult.Grade(spec)
	if err != nil {
		return nil, err
	}
	return judgeResult, nil
}
