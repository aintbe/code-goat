package config

import (
	"fmt"
	"os"
	"strings"

	"github.com/aintbe/code-goat/evaluator/utils"
)

type TestCase struct {
	Id       string
	HasInput bool
}

func FindTestCases(b *Benchmark) ([]*TestCase, error) {
	dirPath := fmt.Sprintf("%s/%s/testcases", b.testDir, b.Problem)

	entries, err := os.ReadDir(dirPath)
	if err != nil {
		return nil, utils.Error(err, "read directory", "")
	}

	testcaseMap := make(map[string]bool)
	for _, entry := range entries {
		parts := strings.Split(entry.Name(), ".")
		if len(parts) == 2 {
			id, ext := parts[0], parts[1]

			// Store testcase and mark if input file is found for it
			hasInput, ok := testcaseMap[id]
			if !ok || !hasInput {
				testcaseMap[id] = ext == "in"
			}
		}
	}

	testcases := make([]*TestCase, 0, len(testcaseMap))
	for id, hasInput := range testcaseMap {
		testcases = append(testcases, &TestCase{
			Id:       id,
			HasInput: hasInput,
		})
	}

	return testcases, nil
}
