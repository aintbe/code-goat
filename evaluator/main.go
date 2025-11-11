package main

import (
	"fmt"
	"log"
	"time"

	"github.com/aintbe/code-goat/evaluator/config"
	"github.com/aintbe/code-goat/evaluator/runner"
	"github.com/aintbe/code-goat/evaluator/summary"
	"github.com/aintbe/code-goat/evaluator/utils"
)

func main() {
	// Parse arguments to get the targeted benchmark.
	benchmark := config.NewBenchmark()

	// Check the configuration of the benchmark.
	profile, err := config.NewProfile(benchmark)
	if err != nil {
		log.Fatalln(err)
	}
	testcases, err := config.FindTestCases(benchmark)
	if err != nil {
		log.Fatalln(err)
	}

	// Define containers for evaluation results.
	summaries := make(map[string]*summary.Summary)
	results := make(utils.Serializable[string, []*runner.JudgeResult])
	resultsPerJudger := make([]*runner.JudgeResult, 0, benchmark.Iteration)

	// Run submitted code for all testcases.
	for _, tc := range testcases {
		// Define a judge spec for the current testcase.
		spec := config.NewJudgeSpec(benchmark, tc, profile)
		summary := summary.NewSummary()
		resultsPerTc := make([]*runner.JudgeResult, 0, benchmark.Iteration*3)

		// Run current testcase for all judgers.
		for judger, run := range runner.Runner {
			resultsPerJudger = resultsPerJudger[:0] // Reset

			// Run a single testcase for configured times to get average result.
			for i := 0; i < benchmark.Iteration; i++ {
				result, err := run(spec)
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
			summary.Add(resultsPerJudger)
			resultsPerTc = append(resultsPerTc, resultsPerJudger...)
		}

		// Store results into map.
		summaries[tc.Id] = summary
		results[tc.Id] = resultsPerTc
	}

	reportName := fmt.Sprintf(
		"%s/%s/%s/%s",
		benchmark.ReportDir,
		benchmark.Problem,
		benchmark.Language,
		benchmark.Submission,
	)

	// Write generated reports into files.
	evaluation := utils.AutoSerializable{
		"benchmark":   benchmark,
		"environment": utils.NewEnvironment(),
		"summaries":   summaries,
	}
	if err = evaluation.Dump(reportName, utils.YAML); err != nil {
		log.Println(err)
	}
	if err = results.Dump(reportName, utils.JSON); err != nil {
		log.Println(err)
	}
}

// todo:
// - 점수 매기기
// - seccomp
// - log 정리
