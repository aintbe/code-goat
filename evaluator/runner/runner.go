package runner

import (
	"github.com/aintbe/code-goat/evaluator/config"
)

type Judger string
const (
	CodeGoat 	Judger = "code_goat"
	QojV1		Judger = Judger(qojV1)
	QojV2		Judger = Judger(qojV2)
)

type JudgerRunner func(s *config.JudgeSpec) (*JudgeResult, error)

func qojWrapper(version qojVersion) JudgerRunner {
    return func(s *config.JudgeSpec) (*JudgeResult, error) {
        return RunQoj(version, s)
    }
}

var Runner = map[Judger]JudgerRunner{
    CodeGoat: RunCodeGoat,
	QojV1: qojWrapper(qojV1),
	QojV2: qojWrapper(qojV2),
}
