package summary

import (
	"time"

	"github.com/aintbe/code-goat/evaluator/runner"
	"github.com/aintbe/code-goat/evaluator/utils"
)

type Summary struct {
	Status    map[runner.Judger]string `yaml:"status"`
	Memory    stats[utils.ByteSize]    `yaml:"memory"`
	CpuTime   stats[time.Duration]     `yaml:"cpu_time"`
	RealTime  stats[time.Duration]     `yaml:"real_time"`
	JudgeTime stats[time.Duration]     `yaml:"judge_time"`
}

type stats[T statType] map[runner.Judger]*stat[T]

func NewSummary() *Summary {
	return &Summary{
		Status:    make(map[runner.Judger]string),
		Memory:    make(stats[utils.ByteSize]),
		CpuTime:   make(stats[time.Duration]),
		RealTime:  make(stats[time.Duration]),
		JudgeTime: make(stats[time.Duration]),
	}
}

func (s *Summary) Add(resultList []*runner.JudgeResult) {
	var status runner.JudgeStatus
	var memory stat[utils.ByteSize]
	var cpuTime stat[time.Duration]
	var realTime stat[time.Duration]
	var judgeTime stat[time.Duration]

	count := len(resultList)

	for _, result := range resultList {
		if status == result.Status || status == "" {
			status = result.Status
		} else {
			status = "Inconsistent"
		}

		if result.ResourceUsage != nil {
			memory.Update(result.ResourceUsage.Memory)
			cpuTime.Update(result.ResourceUsage.CpuTime)
			realTime.Update(result.ResourceUsage.RealTime)
			judgeTime.Update(result.JudgeTime)
		} else {
			count -= 1
		}
	}

	// Add new summary for this judger
	if count > 0 {
		judger := resultList[0].Judger
		s.Status[judger] = string(status)
		s.Memory[judger] = memory.Finalize(count)
		s.CpuTime[judger] = cpuTime.Finalize(count)
		s.RealTime[judger] = realTime.Finalize(count)
		s.JudgeTime[judger] = judgeTime.Finalize(count)
	}
}
