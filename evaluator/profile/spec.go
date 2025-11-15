package profile

import (
	"fmt"

	"github.com/aintbe/code-goat/evaluator/utils"
)

type JudgeSpec struct {
	InputPath      string
	ExpectedOutput []byte
	OutputPath     string
	ErrorPath      string
	LogPath        string
	Config         // Embed
}

func NewJudgeSpec(b *Benchmark, t *TestCase, c *Config) (*JudgeSpec, error) {
	testcase := fmt.Sprintf("%s/%s/testcases/%s", b.TestDir, b.Problem, t.Id)
	inputPath := ""
	if t.HasInput {
		inputPath = testcase + ".in"
	}
	expectedOutput, err := utils.ReadBytes(testcase + ".out")
	if err != nil {
		return nil, err
	}

	targetDir := b.GetTargetDir()
	return &JudgeSpec{
		InputPath:      inputPath,
		ExpectedOutput: expectedOutput,
		OutputPath:     targetDir + t.Id + ".out",
		ErrorPath:      targetDir + t.Id + ".err",
		LogPath:        targetDir + t.Id + ".log",
		Config:         *c,
	}, nil
}
