package utils

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"unicode"
)

func Error(err error, operation, data string) error {
	if data == "" {
		return fmt.Errorf("failed to %s: %w", operation, err)
	}
	return fmt.Errorf(`failed to %s: %w
>>>
%s
<<<`, operation, err, data)
}

func ReadBytes(filePath string) ([]byte, error) {
	bytes, err := os.ReadFile(filePath)
	if err != nil {
		return nil, Error(err, "read file", "")
	}
	return cleanBytes(bytes), nil
}

func cleanBytes(a []byte) []byte {
	sep := []byte("\n")
	b := bytes.Split(bytes.TrimRightFunc(a, unicode.IsSpace), sep)

	for idx, val := range b {
		b[idx] = bytes.TrimRightFunc(val, unicode.IsSpace)
	}
	return bytes.Join(b, sep)
}

func Unmarshal[T interface{}](jsonBytes []byte) (*T, error) {
	var result T

	err := json.Unmarshal(jsonBytes, &result)
	if err != nil {
		return &result, Error(err, "unmarshal", string(jsonBytes))
	}
	return &result, nil
}
