package runner

import "github.com/aintbe/code-goat/evaluator/config"

type Runner interface {
	RunQingdao(s *config.RunSpec) (string, error)
	RunCodeGoat(s *config.RunSpec) (string, error)
}