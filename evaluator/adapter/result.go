package adapter

import (
	"bytes"
	"time"

	"github.com/aintbe/code-goat/evaluator/profile"
	"github.com/aintbe/code-goat/evaluator/utils"
)

type JudgeResult struct {
	Judger        Judger         `json:"judger"`
	JudgeTime     time.Duration  `json:"judge_time"`
	Status        JudgeStatus    `json:"status"`
	Message       *string        `json:"message"`
	ExitCode      *int           `json:"exit_code"`
	Signal        *string        `json:"signal"`
	ResourceUsage *ResourceUsage `json:"resource_usage"`
	Output        *string        `json:"output"`
}

type ResourceUsage struct {
	Memory   uint64 `json:"memory"`
	CpuTime  uint32 `json:"cpu_time"`
	RealTime uint32 `json:"real_time"`
}

type JudgeStatus string

const (
	Exited                JudgeStatus = "Exited"
	Accepted              JudgeStatus = "Accepted"
	WrongAnswer           JudgeStatus = "WrongAnswer"
	CpuTimeLimitExceeded  JudgeStatus = "CpuTimeLimitExceeded"
	RealTimeLimitExceeded JudgeStatus = "RealTimeLimitExceeded"
	MemoryLimitExceeded   JudgeStatus = "MemoryLimitExceeded"
	RuntimeError          JudgeStatus = "RuntimeError"
	InternalError         JudgeStatus = "InternalError"
)

func (j *JudgeResult) Grade(spec *profile.JudgeSpec) error {
	if j.Status != Exited {
		return nil
	}

	output, err := utils.ReadBytes(spec.OutputPath)
	if err != nil {
		return err
	}

	accepted := bytes.Equal(spec.ExpectedOutput, output)
	if accepted {
		j.Status = Accepted
	} else {
		j.Status = WrongAnswer
		outputStr := string(output)
		j.Output = &outputStr
	}

	return nil
}
