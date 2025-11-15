package adapter

import "github.com/aintbe/code-goat/evaluator/profile"

type Judger string

const (
	CodeGoat Judger = "code_goat"
	QojV1    Judger = Judger(qojV1)
	QojV2    Judger = Judger(qojV2)
)

type RunFunc func(s *profile.JudgeSpec) (*JudgeResult, error)

func qojFuncWrapper(version qojVersion) RunFunc {
	return func(s *profile.JudgeSpec) (*JudgeResult, error) {
		return RunQoj(version, s)
	}
}

var JudgerAdapter = map[Judger]RunFunc{
	CodeGoat: RunCodeGoat,
	QojV1:    qojFuncWrapper(qojV1),
	QojV2:    qojFuncWrapper(qojV2),
}
