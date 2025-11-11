package runner

import (
	"encoding/json"
	"time"

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

type ResourceUsage struct {
	Memory   uint64 `json:"memory"`
	CpuTime  uint32 `json:"cpu_time"`
	RealTime uint32 `json:"real_time"`
}

func NewJudgeResult(jsonStr string, judger Judger, startTime time.Time) (*JudgeResult, error) {
	// Total time spent to run judger since execution
	judgeTime := time.Since(startTime)

	var judgeResult JudgeResult
	err := json.Unmarshal([]byte(jsonStr), &judgeResult)
	if err != nil {
		return &judgeResult, utils.Error(err, "unmarshal", jsonStr)
	}

	judgeResult.Judger = judger
	judgeResult.JudgeTime = judgeTime
	return &judgeResult, nil
}
