package utils

import (
	"encoding/json"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

type Serializable[K string, V any] map[K]V
type AutoSerializable = Serializable[string, interface{}]

type Extension string

const (
	YAML Extension = ".yaml"
	JSON Extension = ".json"
)

func (s Serializable[K, T]) Dump(fileName string, ext Extension) error {
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
