package profile

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

func LoadTestCases(b *Benchmark) ([]*TestCase, error) {
	dirPath := fmt.Sprintf("%s/%s/testcases", b.TestDir, b.Problem)

	entries, err := os.ReadDir(dirPath)
	if err != nil {
		return nil, utils.Error(err, "read directory", "")
	}

	testcaseMap := make(map[string]bool)
	for _, entry := range entries {
		tokens := strings.Split(entry.Name(), ".")
		if len(tokens) == 2 {
			id, ext := tokens[0], tokens[1]

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
