package runner

import (
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"text/template"
)

type JudgeResult struct {
	Status 			string			`json:"status"`
	Message 		string			`json:"message"`
	ExitCode 		int				`json:"exit_code"`
	Signal			string			`json:"signal"`
	ResourceUsage 	ResourceUsage	`json:"resource_usage"`
}

type ResourceUsage struct {
	Memory 		uint64	`json:"memory"`
    CpuTime 	int64	`json:"cpu_time"`
    RealTime	big.Int	`json:"real_time"`
}

func (j *JudgeResult) Printf() {
	const tmpl = `
┌───────────┬─────────────────────────────────────────────────────┐
│ Status    │ {{printf "%-51s" .Status}} │
├───────────┼─────────────────────────────────────────────────────┤
│ Exit Code │ {{printf "%-51d" .ExitCode}} │
├───────────┼─────────────────────────────────────────────────────┤
│ Signal    │ {{printf "%-51s" .Signal}} │
├───────────┼─────────────────────────────────────────────────────┤
│ Usage     │ Memory          │ Cpu Time        │ Real Time       │
│           ├─────────────────┼─────────────────┼─────────────────┤
│           │ {{printf "%-15d" .ResourceUsage.Memory}} │ {{printf "%-15d" .ResourceUsage.CpuTime}} │ {{printf "%-15d" .ResourceUsage.RealTime}} │
└───────────┴─────────────────┴─────────────────┴─────────────────┘
Message: {{.Message}}
`
	t, err := template.New("JudgeResult").Parse(tmpl)
    if err != nil {
        panic(err)
    }
    t.Execute(os.Stdout, j)
}

func NewJudgeResult(jsonStr string) (JudgeResult, error) {
	var judgeResult JudgeResult
	
	err := json.Unmarshal([]byte(jsonStr), &judgeResult)
    if err != nil {
        return judgeResult, fmt.Errorf("failed to unmarshal judge result: %w", err)
    }
	return judgeResult, nil
}
