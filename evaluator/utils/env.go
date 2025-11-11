package utils

import (
	"fmt"
	"runtime"
	"time"

	"github.com/pbnjay/memory"
	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/host"
)

type Environment struct {
	Os      	string 		`yaml:"os"`
	Cpu      	string 		`yaml:"cpu"`
	Memory   	string 		`yaml:"memory"`
    Timestamp 	time.Time	`yaml:"timestamp"`
}

func GetEnvironment() Environment {
	os := fmt.Sprintf("%s/%s", runtime.GOOS, runtime.GOARCH)
	hostInfo, err := host.Info()
	if err == nil {
		os = fmt.Sprintf("%s (%s %s)", os, hostInfo.Platform, hostInfo.PlatformVersion)
	}

	return Environment{
        Timestamp: time.Now(),
		Os:       os,
		Cpu:      getCpuModel(),
		Memory: fmt.Sprintf("%.1fGB", float64(memory.TotalMemory()) / (1024 * 1024 * 1024)),
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
	return fmt.Sprintf("%s %s (%d Core)", info[0].VendorID, modelName, runtime.NumCPU())
}
