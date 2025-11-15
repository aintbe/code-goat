package main

import (
	"fmt"
	"log"
	"time"

	"github.com/aintbe/code-goat/evaluator/adapter"
	"github.com/aintbe/code-goat/evaluator/eval"
	"github.com/aintbe/code-goat/evaluator/profile"
	"github.com/aintbe/code-goat/evaluator/utils"
)

func main() {
	// Parse arguments to get the targeted benchmark.
	benchmark := profile.LoadBenchmark()

	// Check the configuration of the benchmark.
	config, err := profile.LoadConfig(benchmark)
	if err != nil {
		log.Fatalln(err)
	}
	testcases, err := profile.LoadTestCases(benchmark)
	if err != nil {
		log.Fatalln(err)
	}

	// Define collections of evaluation results.
	results := make(utils.Serializable[string, []*adapter.JudgeResult])
	resultsPerJudger := make([]*adapter.JudgeResult, 0, benchmark.Iteration)
	evaluations := make(map[string]*eval.Evaluation)

	// Run submitted code for all testcases.
	for _, tc := range testcases {
		spec, err := profile.NewJudgeSpec(benchmark, tc, config)
		if err != nil {
			log.Println(err) // Do not stop the entire evaluation.
			continue
		}

		evaluation := eval.NewEvaluation()
		resultsPerTc := make([]*adapter.JudgeResult, 0, benchmark.Iteration*3)

		// Run current testcase for all judgers.
		for judger, runJudger := range adapter.JudgerAdapter {
			resultsPerJudger = resultsPerJudger[:0] // Reset

			// Run a single testcase for configured times to get average result.
			for i := 0; i < benchmark.Iteration; i++ {
				result, err := runJudger(spec)

				if err != nil {
					log.Printf("- [%s] %s\n", judger, err)
				} else {
					resultsPerJudger = append(resultsPerJudger, result)
				}

				// Release control long enough after each run so that we can
				// check the worst case scenario in terms of judge time.
				// CodeGoat would run quicker by tens of ms in an actual environment
				// than it does in this evaluator system.
				time.Sleep(10 * time.Millisecond)
			}
			evaluation.Add(resultsPerJudger)
			resultsPerTc = append(resultsPerTc, resultsPerJudger...)
		}

		// Store outputs into map.
		evaluations[tc.Id] = evaluation
		results[tc.Id] = resultsPerTc
	}

	reportName := fmt.Sprintf(
		"%s/%s/%s/%s",
		benchmark.ReportDir,
		benchmark.Problem,
		benchmark.Language,
		benchmark.Submission,
	)

	// Write generated evaluations and results into files.
	evaluation := utils.AutoSerializable{
		"benchmark":   benchmark,
		"environment": utils.NewEnvironment(),
		"limits":      config.ResourceLimit,
		"tests":       evaluations,
	}
	if err = evaluation.Dump(reportName, utils.Yaml); err != nil {
		log.Println(err)
	}
	if err = results.Dump(reportName, utils.Json); err != nil {
		log.Println(err)
	}
}

// todo:
// - 점수 매기기 v
// - 결과 정확한지 확인하기 (status, time, memory 등)
// - seccomp - java,
// - seccomp - python running (일단 python 먼저)
// v - log 정리, 자녀도 그대로 로깅 가능한지 확인하기
