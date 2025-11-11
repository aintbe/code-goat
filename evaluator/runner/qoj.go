package runner

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"time"

	"github.com/aintbe/code-goat/evaluator/config"
	"github.com/aintbe/code-goat/evaluator/utils"
)

type qojVersion string

const (
	qojV1 qojVersion = "qoj_v1"
	qojV2 qojVersion = "qoj_v2"
)

func RunQoj(version qojVersion, s *config.JudgeSpec) (*JudgeResult, error) {
	var binaryPath string
	if version == qojV1 {
		binaryPath = "QOJ_LIBJUDGER_PATH"
	} else {
		binaryPath = "QOJ_LIBJUDGER_V2_PATH"
	}

	startTime := time.Now()
	cmd := convertToCmd(s, os.Getenv(binaryPath))

	// var stdin bytes.Buffer
	var stdout bytes.Buffer
	// var stderr bytes.Buffer
	// cmd.Stdin = &stdin
	cmd.Stdout = &stdout
	// cmd.Stderr = &stderr
	// stdin.Write(input)

	err := cmd.Run()
	if err != nil {
		return &JudgeResult{}, utils.Error(err, "execute", cmd.String())
	}

	// Total time spent to run judger since execution
	judgeTime := time.Since(startTime)

	qojResult, err := newQojJudgeResult(stdout.Bytes())
	if err != nil {
		return &JudgeResult{}, err
	}

	return qojResult.generalize(version, judgeTime)
}

const (
	MaxCpuTime           = "--max_cpu_time="
	MaxRealTime          = "--max_real_time="
	MaxMemory            = "--max_memory="
	MaxStackSize         = "--max_stack="
	MaxProcessNumber     = "--max_process_number="
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

func convertToCmd(s *config.JudgeSpec, binaryPath string) *exec.Cmd {
	var args utils.StringSlice

	args.AddString(ExePath, s.ExePath)
	args.AddString(InputPath, s.InputPath)
	args.AddString(OutputPath, s.OutputPath)
	args.AddString(ErrorPath, s.ErrorPath)
	// TODO: support logging & seccomp rule set
	// args.AddString(LogPath, s.logPath)
	// args.AddString(SeccompRuleName, )

	args.Extend(Args, s.Args)
	s.Envs.AddString("PATH=", os.Getenv("PATH"))
	args.Extend(Env, s.Envs)

	args.AddUInt(MaxMemory, uint64(s.ResourceLimit.Memory))
	args.AddUInt(MaxCpuTime, s.ResourceLimit.CpuTime)
	args.AddUInt(MaxRealTime, uint64(s.ResourceLimit.RealTime))
	args.AddUInt(MaxStackSize, s.ResourceLimit.Stack)
	args.AddUInt(MaxProcessNumber, s.ResourceLimit.NProcess)
	args.AddUInt(MaxOutputSize, s.ResourceLimit.Output)

	return exec.Command(binaryPath, args...)
}

type qojJudgeResult struct {
	CpuTime    int `json:"cpuTime"`
	RealTime   int `json:"realTime"`
	Memory     int `json:"memory"`
	Signal     int `json:"signal"`
	ErrorCode  int `json:"exitCode"`
	ExitCode   int `json:"errorCode"`
	ResultCode int `json:"resultCode"`
}

var statusMap = map[int]JudgeStatus{
	0: Exited,
	1: CpuTimeLimitExceeded,
	2: RealTimeLimitExceeded,
	3: MemoryLimitExceeded,
	4: RuntimeError,
	5: InternalError,
}

func newQojJudgeResult(jsonBytes []byte) (*qojJudgeResult, error) {
	var judgeResult qojJudgeResult

	err := json.Unmarshal(jsonBytes, &judgeResult)
	if err != nil {
		return &judgeResult, utils.Error(err, "unmarshal", string(jsonBytes))
	}
	return &judgeResult, nil
}

func (q *qojJudgeResult) generalize(version qojVersion, judgeTime time.Duration) (*JudgeResult, error) {
	signal := fmt.Sprint(q.Signal)

	return &JudgeResult{
		Judger:    Judger(version),
		JudgeTime: judgeTime,
		Status:    statusMap[q.ResultCode],
		Message:   nil,
		ExitCode:  &q.ExitCode,
		Signal:    &signal,
		ResourceUsage: &ResourceUsage{
			Memory:   uint64(q.Memory),
			CpuTime:  uint32(q.CpuTime),
			RealTime: uint32(q.RealTime),
		},
	}, nil
}
