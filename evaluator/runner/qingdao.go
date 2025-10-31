package runner

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"time"

	"github.com/aintbe/code-goat/evaluator/config"
)

func timeTrack(start time.Time, name string) {
	// 이 함수는 defer에 의해 호출되므로,
	// 감싸고 있는 함수가 종료될 때 실행됩니다.
	elapsed := time.Since(start)
	fmt.Printf("%s 실행 시간: %s\n", name, elapsed)
}

func RunQingdao(s *config.RunSpec) (string, error) {
	defer timeTrack(time.Now(), "qingdao1")

	cmd := convertToCmd(s)
	fmt.Printf("===cmd===\n%s\n", cmd.String())
	
	
	// var stdin bytes.Buffer
	var stdout bytes.Buffer
	// var stderr bytes.Buffer
	// cmd.Stdin = &stdin
	cmd.Stdout = &stdout
	// cmd.Stderr = &stderr
	// stdin.Write(input)

	defer timeTrack(time.Now(), "qingdao2")

	err := cmd.Run()
	if err != nil {
		return "{}", fmt.Errorf("sandbox execution failed: %w", err)
	}
	
	return stdout.String(), nil	
}

const (
	MaxCpuTime           = "--max_cpu_time="
	MaxRealTime          = "--max_real_time="
	MaxMemory            = "--max_memory="
	MaxStackSize         = "--max_stack="
	MaxProcessNumber	 = "--max_process_number="
	MaxOutputSize        = "--max_output_size="
	ExePath              = "--exe_path="
	InputPath            = "--input_path="
	OutputPath           = "--output_path="
	ErrorPath            = "--error_path="
	LogPath              = "--log_path="
	Args                 = "--args="
	Env                  = "--env="
	SeccompRuleName      = "--seccomp_rule_name="
	MemoryLimitCheckOnly = "--memory_limit_check_only="
	Uid                  = "--uid="
	Gid                  = "--gid="
)

func convertToCmd(s *config.RunSpec) *exec.Cmd {
	var args config.StringSlice

	args.AddString(ExePath, s.ExePath)
	args.AddString(InputPath, s.InputPath)
	args.AddString(OutputPath, s.OutputPath)
	args.AddString(ErrorPath, s.ErrorPath)
	// TODO: support logging & seccomp rule set
	// args.AddString(LogPath, s.logPath)
	// args.AddString(SeccompRuleName, )

	args.Extend(Args, s.Args)
	s.Envs.Concat("PATH=", os.Getenv("PATH"))
	args.Extend(Env, s.Envs)

	args.AddUInt(MaxMemory, uint64(s.ResourceLimit.Memory))
	args.AddUInt(MaxCpuTime, s.ResourceLimit.CpuTime)
	args.AddUInt(MaxRealTime, uint64(s.ResourceLimit.RealTime))
	args.AddUInt(MaxStackSize, s.ResourceLimit.Stack)
	args.AddUInt(MaxProcessNumber, s.ResourceLimit.NProcess)
	args.AddUInt(MaxOutputSize, s.ResourceLimit.Output)
	
	binaryPath := os.Getenv("QINGDAO_LIBJUDGER_PATH")
	return exec.Command(binaryPath, args...)
}


type ExecResult struct {
	CpuTime    *int        `json:"cpuTime"`
	RealTime   *int        `json:"realTime"`
	Memory     *int        `json:"memory"`
	Signal     *int        `json:"signal"`
	ErrorCode  *int        `json:"exitCode"`
	ExitCode   *int        `json:"errorCode"`
	ResultCode *ResultCode `json:"resultCode"`
}

type ResultCode int8

// libjudger의 정의값
const ( // ResultCode
	RUN_SUCCESS ResultCode = 0 + iota // this only means the process exited normally
	CPU_TIME_LIMIT_EXCEEDED
	REAL_TIME_LIMIT_EXCEEDED
	MEMORY_LIMIT_EXCEEDED
	RUNTIME_ERROR
	SYSTEM_ERROR
)


