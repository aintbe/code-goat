package summary

import (
	"math/big"
	"time"

	"github.com/aintbe/code-goat/evaluator/runner"
)

type Summary struct {
	Status		map[runner.Judger]string	`yaml:"status"`
	Memory 		stats[uint64]				`yaml:"memory"`
    CpuTime 	stats[int64]				`yaml:"cpu_time"`
    RealTime	stats[*big.Int]				`yaml:"real_time"`
	JudgeTime 	stats[time.Duration]		`yaml:"judge_time"`
}

type stats[T statType] map[runner.Judger]*stat[T]

func NewSummary() *Summary {
    return &Summary{
        Status:     make(map[runner.Judger]string),
        Memory:     make(stats[uint64]),
        CpuTime:    make(stats[int64]),
        RealTime:   make(stats[*big.Int]),
		JudgeTime:	make(stats[time.Duration]),
    }
}

func (s *Summary) Add(resultList []*runner.JudgeResult) {
	var status runner.JudgeStatus
	var memory stat[uint64]
	var cpuTime stat[int64]
	realTime := stat[*big.Int] {
		sum: big.NewInt(0),
		Avg: big.NewInt(0),
		Max: big.NewInt(0),
		Min: big.NewInt(0),
	}
	var judgeTime stat[time.Duration]
	
	count := int64(len(resultList))

	for _, result := range resultList {
		if status == result.Status || status == "" {
			status = result.Status
		} else {
			status = "Inconsistent"
		}
		
		if result.ResourceUsage != nil {
			memory.Update(result.ResourceUsage.Memory)
			cpuTime.Update(result.ResourceUsage.CpuTime)
			realTime.Update(&result.ResourceUsage.RealTime)
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

// {
//   "results": [
//     {
//       "program_name": "ProgramA",
//       "test_case_id": "TC001",
//       "measurements": [
//         {
//           "metric": "ExecutionTime",
//           "unit": "ms",
//           "value": 450.7,
//           "runs": [448.2, 451.1, 452.8] // <--- Program A의 모든 시간 실행 결과
//         },
//         {
//           "metric": "PeakMemory",
//           "unit": "MB",
//           "value": 128.5,
//           "runs": [128.5, 128.0, 129.0] // <--- Program A의 모든 메모리 실행 결과
//         }
//       ]
//     },
//     {
//       "program_name": "ProgramB",
//       // ... Program B에 대한 모든 정보 ...
//     }
//   ]
// }