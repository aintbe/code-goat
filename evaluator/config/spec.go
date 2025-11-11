package config

import (
	"fmt"
)

type JudgeSpec struct {
	ExePath    string
	InputPath  string
	OutputPath string
	ErrorPath  string
	AnswerPath string
	Profile    // Embed
}

func NewJudgeSpec(b *Benchmark, t *TestCase, p *Profile) *JudgeSpec {
	targetDir := fmt.Sprintf("%s/%s/%s/%s/", b.testDir, b.Problem, b.Submission, b.Language)
	inputPath := fmt.Sprintf("%s/%s/testcases/%s.in", b.testDir, b.Problem, t.Id)

	return &JudgeSpec{
		ExePath:    targetDir + "main",
		InputPath:  inputPath,
		OutputPath: targetDir + t.Id + ".out",
		ErrorPath:  targetDir + t.Id + ".err",
		AnswerPath: "", // TODO: Do not grade
		Profile:    *p,
	}
}
