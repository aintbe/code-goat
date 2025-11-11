package utils

import "fmt"

func Error(err error, operation, data string) error {
	if data == "" {
		return fmt.Errorf("failed to %s: %w", operation, err)
	}
	return fmt.Errorf(`failed to %s: %w
>>>
%s
<<<`, operation, err, data)	
}