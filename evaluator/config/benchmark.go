package config

import (
	"flag"
)

type Benchmark struct {
	Problem    string `yaml:"problem"`
	Submission string `yaml:"submission"`
	Language   string `yaml:"language"`
	testDir    string
	ReportDir  string `yaml:"-"`
}

func NewBenchmark() *Benchmark {
	var benchmark Benchmark

	// Define all the flags to parse from CLI arguments.
	flag.StringVar(&benchmark.Problem, "problem", "", "Problem ID.")
	flag.StringVar(&benchmark.Submission, "submission", "", "Submission ID.")
	flag.StringVar(&benchmark.Language, "language", "", "Language in which the submitted code was written.")
	flag.StringVar(&benchmark.testDir, "test-dir", "/workspace/tests", "Absolute path to the tests dir.")
	flag.StringVar(&benchmark.ReportDir, "report-dir", "/workspace/reports", "Absolute path to output dir.")

	flag.Parse()
	return &benchmark
}
