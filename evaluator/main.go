package main

import (
	"encoding/json"
	"fmt"

	"github.com/aintbe/code-goat/evaluator/config"
	"github.com/aintbe/code-goat/evaluator/runner"
)

// func timeTrack(start time.Time, name string) {
// 	// 함수가 종료될 때까지 대기했다가 실행됨
// 	elapsed := time.Since(start)
// 	fmt.Printf("%s 실행 시간: %s\n", name, elapsed)
// }

func main() {
	spec := config.NewRunSpec()
	// fmt.Printf("%#v\n", spec)

	data1, err := runner.RunQingdao(spec);
	if err != nil {
		fmt.Printf("\n%s\n", err)
	}

	a, err := NewJudgeResult(data1)
	if err != nil {
		fmt.Printf("\n%s\n", err)
	} else {
		fmt.Printf("Qingdao:\n%#v\n", a)
		fmt.Printf("%s\n\n", data1)
	}
	
	data2, err := runner.RunCodeGoat(spec);
	if err != nil {
		fmt.Printf("\n%s\n", err)
	}
	fmt.Printf("Code Goat:\n%s\n", data2)
	// b, err := NewJudgeResult(data2)
	// if err != nil {
	// 	fmt.Printf("\n%s\n", err)
	// } else {
	// 	fmt.Printf("Code Goat:\n%#v\n", b)
	// 	fmt.Printf("%s\n\n", data2)
	// }
}

type JudgeResult struct {
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

func NewJudgeResult(jsonStr string) (JudgeResult, error) {
	var judgeResult JudgeResult
	
	err := json.Unmarshal([]byte(jsonStr), &judgeResult)
    if err != nil {
        return judgeResult, fmt.Errorf("failed to unmarshal judge result: %w", err)
    }
	return judgeResult, nil
}
