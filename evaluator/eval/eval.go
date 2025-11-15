package eval

import (
	"time"

	"github.com/aintbe/code-goat/evaluator/adapter"
	"github.com/aintbe/code-goat/evaluator/types"
)

type Evaluation struct {
	Status    map[adapter.Judger]string `yaml:"status"`
	Memory    statMap[types.ByteSize]   `yaml:"memory"`
	CpuTime   statMap[time.Duration]    `yaml:"cpu_time"`
	RealTime  statMap[time.Duration]    `yaml:"real_time"`
	JudgeTime statMap[time.Duration]    `yaml:"judge_time"`
}

type statMap[T statType] map[adapter.Judger]*stat[T]

func NewEvaluation() *Evaluation {
	return &Evaluation{
		Status:    make(map[adapter.Judger]string),
		Memory:    make(statMap[types.ByteSize]),
		CpuTime:   make(statMap[time.Duration]),
		RealTime:  make(statMap[time.Duration]),
		JudgeTime: make(statMap[time.Duration]),
	}
}

func (s *Evaluation) Add(resultList []*adapter.JudgeResult) {
	var status adapter.JudgeStatus
	var memory stat[types.ByteSize]
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

	// Add new evaluation for this judger
	if count > 0 {
		judger := resultList[0].Judger
		s.Status[judger] = string(status)
		s.Memory[judger] = memory.Finalize(count)
		s.CpuTime[judger] = cpuTime.Finalize(count)
		s.RealTime[judger] = realTime.Finalize(count)
		s.JudgeTime[judger] = judgeTime.Finalize(count)
	}
}
