package config

import (
	"fmt"
	"os"

	"github.com/aintbe/code-goat/evaluator/utils"
	"gopkg.in/yaml.v3"
)

type Profile struct {
	Args          utils.StringSlice `yaml:"args"`
	Envs          utils.StringSlice `yaml:"envs"`
	ResourceLimit ResourceLimit     `yaml:"memory"`
}

type ResourceLimit struct {
	Memory   uint32 `yaml:"memory"`
	CpuTime  uint64 `yaml:"cpu_time"`
	RealTime uint32 `yaml:"real_time"`
	Stack    uint64 `yaml:"stack"`
	NProcess uint64 `yaml:"n_process"`
	Output   uint64 `yaml:"output"`
}

func NewProfile(b *Benchmark) (*Profile, error) {
	filePath := fmt.Sprintf("%s/%s/profile.yaml", b.testDir, b.Problem)

	yamlBytes, err := os.ReadFile(filePath)
	if err != nil {
		return nil, utils.Error(err, "open profile", "")
	}

	var profile Profile
	err = yaml.Unmarshal(yamlBytes, &profile)
	if err != nil {
		return &profile, utils.Error(err, "unmarshal", string(yamlBytes))
	}

	return &profile, nil
}
