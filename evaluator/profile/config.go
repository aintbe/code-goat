package profile

import (
	"fmt"
	"os"

	"github.com/aintbe/code-goat/evaluator/constants"
	"github.com/aintbe/code-goat/evaluator/types"
	"github.com/aintbe/code-goat/evaluator/utils"
	"gopkg.in/yaml.v3"
)

type Config struct {
	ExePath       string
	ScmpPolicy    constants.ScmpPolicy
	Args          types.StringSlice `yaml:"args"`
	Envs          types.StringSlice `yaml:"envs"`
	ResourceLimit ResourceLimit     `yaml:"limit"`
}

type ResourceLimit struct {
	Memory   uint64 `yaml:"memory"`
	CpuTime  uint32 `yaml:"cpu_time"`
	RealTime uint32 `yaml:"real_time"`
	Stack    uint32 `yaml:"stack"`
	NProcess uint16 `yaml:"n_process"`
	Output   uint32 `yaml:"output"`
}

func LoadConfig(b *Benchmark) (*Config, error) {
	filePath := fmt.Sprintf("%s/%s/config.yaml", b.TestDir, b.Problem)

	yamlBytes, err := os.ReadFile(filePath)
	if err != nil {
		return nil, utils.Error(err, "open config", "")
	}

	var config Config
	err = yaml.Unmarshal(yamlBytes, &config)
	if err != nil {
		return nil, utils.Error(err, "unmarshal", string(yamlBytes))
	}

	// Adjust the config according to the configured language.
	err = config.adjustToLanguage(b.Language, b.GetTargetDir())

	return &config, err
}

func (c *Config) adjustToLanguage(language constants.Language, targetDir string) error {
	switch language {
	case constants.C, constants.Cpp:
		c.ExePath = targetDir + "main"
		c.ScmpPolicy = constants.ScmpStrict

	case constants.Python:
		c.ExePath = "/usr/bin/python3"
		c.Args = append(types.StringSlice{targetDir + "__pycache__/main.cpython-312.pyc"}, c.Args...)
		c.ScmpPolicy = constants.ScmpPython

		fmt.Printf("+ Original memory limit: %d bytes\n", c.ResourceLimit.Memory)
		c.ResourceLimit.Memory = (2*c.ResourceLimit.Memory + 32*constants.MiB)
		fmt.Printf("+ Adjusted memory limit for Python: %d bytes\n", c.ResourceLimit.Memory)
		c.ResourceLimit.CpuTime = 3*c.ResourceLimit.CpuTime + 2
		c.ResourceLimit.RealTime = 3*c.ResourceLimit.RealTime + 2

	default:
		return fmt.Errorf("unsupported language: %s", language)
	}
	return nil
}
