package utils

import (
	"fmt"
	"runtime"

	"github.com/pbnjay/memory"
	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/host"
)

type Environment struct {
	Os        string `yaml:"os"`
	Cpu       string `yaml:"cpu"`
	CoreCount uint8  `yaml:"core_count"`
	Memory    string `yaml:"memory"`
}

func NewEnvironment() Environment {
	os := fmt.Sprintf("%s/%s", runtime.GOOS, runtime.GOARCH)
	hostInfo, err := host.Info()
	if err == nil {
		os = fmt.Sprintf("%s (%s %s)", os, hostInfo.Platform, hostInfo.PlatformVersion)
	}

	return Environment{
		Os:        os,
		Cpu:       getCpuModel(),
		CoreCount: uint8(runtime.NumCPU()),
		Memory:    fmt.Sprintf("%.1fGB", float64(memory.TotalMemory())/(1024*1024*1024)),
	}
}

func getCpuModel() string {
	info, err := cpu.Info()
	if err != nil || len(info) <= 0 {
		return "Unknown"
	}

	modelName := info[0].ModelName
	if modelName == "" {
		modelName = "Chip"
	}
	return fmt.Sprintf("%s %s", info[0].VendorID, modelName)
}
