package profile

import (
	"flag"
	"fmt"

	"github.com/aintbe/code-goat/evaluator/constants"
)

type Benchmark struct {
	Problem    string             `yaml:"problem"`
	Submission string             `yaml:"submission"`
	Language   constants.Language `yaml:"language"`
	Iteration  int                `yaml:"iteration"`
	TestDir    string             `yaml:"-"`
	ReportDir  string             `yaml:"-"`
}

func LoadBenchmark() *Benchmark {
	var benchmark Benchmark
	var language string

	// Define all the flags to parse from CLI arguments.
	flag.StringVar(&benchmark.Problem, "problem", "", "Problem ID.")
	flag.StringVar(&benchmark.Submission, "submission", "", "Submission ID.")
	flag.StringVar(&language, "language", "", "Language in which the submitted code was written.")
	flag.IntVar(&benchmark.Iteration, "iteration", 1, "Iteration count for repeated benchmark execution.")
	flag.StringVar(&benchmark.TestDir, "test-dir", "/workspace/tests", "Absolute path to the tests dir.")
	flag.StringVar(&benchmark.ReportDir, "report-dir", "/workspace/test-reports", "Absolute path to output dir.")

	flag.Parse()

	benchmark.Language = constants.Language(language)
	return &benchmark
}

func (b *Benchmark) GetTargetDir() string {
	return fmt.Sprintf("%s/%s/%s/%s/", b.TestDir, b.Problem, b.Submission, b.Language)
}
