package utils

import (
	"encoding/json"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

type Serializable[T any] map[string]T

type extension string

const (
	YAML extension = ".yaml"
	JSON extension = ".json"
)

func (s Serializable[T]) Dump(fileName string, ext extension) error {
	var bytes []byte
	var err error

	switch ext {
	case YAML:
		bytes, err = yaml.Marshal(s)
	case JSON:
		bytes, err = json.MarshalIndent(s, "", "  ")
	}
	if err != nil {
		return Error(err, "marshal", "")
	}

	filePath := fileName + string(ext)
	if err := os.MkdirAll(filepath.Dir(filePath), 0755); err != nil {
		return Error(err, "create directories", "")
	}
	if err = os.WriteFile(filePath, bytes, 0644); err != nil {
		return Error(err, "write into file", "")
	}

	return nil
}
