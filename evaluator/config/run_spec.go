package config

import (
	"flag"
)

type RunSpec struct {
	ExePath       string
	InputPath     string
	OutputPath    string
	ErrorPath     string
	AnswerPath    string
	Args          StringSlice
	Envs          StringSlice
	ResourceLimit ResourceLimit
}

type ResourceLimit struct {
	Memory    uint32
	CpuTime   uint64
	RealTime  uint32
	Stack     uint64
	NProcess  uint64
	Output    uint64
}

func NewRunSpec() *RunSpec {
	var spec RunSpec
    
	// Define all the flags to parse from CLI arguments.
	flag.StringVar(&spec.ExePath, "exe-path", "", "Absolute path to the executable file.")
	flag.StringVar(&spec.InputPath, "input-path", "", "Absolute path to the input file (for stdin redirection).")
	flag.StringVar(&spec.OutputPath, "output-path", "", "Absolute path to the output file (for stdout redirection).")
	flag.StringVar(&spec.ErrorPath, "error-path", "", "Absolute path to the error file (for stderr redirection).")
	flag.StringVar(&spec.AnswerPath, "answer-path", "", "Absolute path to the error file (for stderr redirection).")
	
	spec.Args, spec.Envs = []string{}, []string{}
	flag.Var(&spec.Args, "args", "List of arguments to pass to the program.")
	flag.Var(&spec.Envs, "envs", "Environment variables to set for the program.")

	var memoryLimitUint64, realTimeLimitUint64 uint64
	flag.Uint64Var(&memoryLimitUint64, "memory-limit", 0, "Peak memory usage in bytes.")
	flag.Uint64Var(&spec.ResourceLimit.CpuTime, "cpu-time-limit", 0, "CPU time used in milliseconds.")
	flag.Uint64Var(&realTimeLimitUint64, "real-time-limit", 0, "Real time used in milliseconds.")
	flag.Uint64Var(&spec.ResourceLimit.Stack, "stack-limit", 0, "Upper limit to stack size in bytes.")
	flag.Uint64Var(&spec.ResourceLimit.NProcess, "n-process-limit", 0, "Maximum number of process.")
	flag.Uint64Var(&spec.ResourceLimit.Output, "output-limit", 0, "Upper limit to output size in bytes.")
	
	// Parse and assign pre-defined flags.
	flag.Parse()
	spec.ResourceLimit.Memory = uint32(memoryLimitUint64)
	spec.ResourceLimit.RealTime = uint32(realTimeLimitUint64)

	return &spec
}

