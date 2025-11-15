package adapter

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"time"

	"github.com/aintbe/code-goat/evaluator/constants"
	"github.com/aintbe/code-goat/evaluator/profile"
	"github.com/aintbe/code-goat/evaluator/types"
	"github.com/aintbe/code-goat/evaluator/utils"
)

type qojVersion string

const (
	qojV1 qojVersion = "qoj_v1"
	qojV2 qojVersion = "qoj_v2"
)

func RunQoj(version qojVersion, s *profile.JudgeSpec) (*JudgeResult, error) {
	var binaryPath string
	if version == qojV1 {
		binaryPath = "QOJ_LIBJUDGER_PATH"
	} else {
		binaryPath = "QOJ_LIBJUDGER_V2_PATH"
	}

	startTime := time.Now()
	cmd := convertToCmd(s, os.Getenv(binaryPath))

	var stdout bytes.Buffer
	// todo: capture stderr for debugging??
	// 이건 dup이 있는데 왜 해야 되는 거임??
	// var stderr bytes.Buffer
	cmd.Stdout = &stdout
	// cmd.Stderr = &stderr

	// Run judger binary.
	err := cmd.Run()
	if err != nil {
		return nil, utils.Error(err, "execute", cmd.String())
	}
	elapsedTime := time.Since(startTime)

	// Parse judge result from output.
	res, err := utils.Unmarshal[qojJudgeResult](stdout.Bytes())
	if err != nil {
		return nil, err
	}
	return res.toJudgeResult(s, version, elapsedTime)
}

const (
	maxCpuTime           = "--max_cpu_time="
	maxRealTime          = "--max_real_time="
	maxMemory            = "--max_memory="
	maxStackSize         = "--max_stack="
	maxProcessNumber     = "--max_process_number="
	maxOutputSize        = "--max_output_size="
	exePath              = "--exe_path="
	inputPath            = "--input_path="
	outputPath           = "--output_path="
	errorPath            = "--error_path="
	logPath              = "--log_path="
	args                 = "--args="
	env                  = "--env="
	seccompRuleName      = "--seccomp_rule_name="
	memoryLimitCheckOnly = "--memory_limit_check_only="
	uid                  = "--uid="
	gid                  = "--gid="
)

var seccompRuleMap = map[constants.ScmpPolicy]string{
	constants.ScmpUnsafe: "",
	constants.ScmpStrict: "c_cpp",
	constants.ScmpPython: "general",
}

func convertToCmd(s *profile.JudgeSpec, binaryPath string) *exec.Cmd {
	var cmdArgs types.StringSlice

	cmdArgs.Add(exePath, s.ExePath)
	cmdArgs.Add(inputPath, s.InputPath)
	cmdArgs.Add(outputPath, s.OutputPath)
	cmdArgs.Add(errorPath, s.ErrorPath)
	cmdArgs.Add(logPath, s.LogPath)
	cmdArgs.Add(seccompRuleName, seccompRuleMap[s.ScmpPolicy])

	cmdArgs.Extend(args, s.Args)
	cmdEnvs := append(s.Envs, "PATH="+os.Getenv("PATH"))
	cmdArgs.Extend(env, cmdEnvs)

	cmdArgs.Add(maxMemory, toString(s.ResourceLimit.Memory))
	cmdArgs.Add(maxCpuTime, toString(s.ResourceLimit.CpuTime))
	cmdArgs.Add(maxRealTime, toString(s.ResourceLimit.RealTime))
	cmdArgs.Add(maxStackSize, toString(s.ResourceLimit.Stack))
	cmdArgs.Add(maxProcessNumber, toString(s.ResourceLimit.NProcess))
	cmdArgs.Add(maxOutputSize, toString(s.ResourceLimit.Output))

	return exec.Command(binaryPath, cmdArgs...)
}

func toString[T uint32 | uint64 | uint16](value T) string {
	if value == 0 {
		return ""
	}
	return strconv.FormatUint(uint64(value), 10)
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

func (q *qojJudgeResult) toJudgeResult(spec *profile.JudgeSpec, version qojVersion, judgeTime time.Duration) (*JudgeResult, error) {
	signal := fmt.Sprint(q.Signal)
	judgeResult := JudgeResult{
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
	}

	err := judgeResult.Grade(spec)
	if err != nil {
		return nil, err
	}
	return &judgeResult, nil
}
