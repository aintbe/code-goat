package types

import (
	"bytes"
	"strings"
)

type StringSlice []string

func (s *StringSlice) Set(value string) error {
	if value != "" {
		*s = append(*s, value)
	}
	return nil
}

func (s *StringSlice) String() string {
	return strings.Join(*s, " ")
}

func (s *StringSlice) Add(key string, value string) {
	if value != "" {
		s.concat(key, value)
	}
}

func (s *StringSlice) Extend(key string, values []string) {
	for _, value := range values {
		s.concat(key, value)
	}
}

func (s *StringSlice) concat(key string, value string) {
	var b bytes.Buffer
	b.WriteString(key)
	b.WriteString(value)
	*s = append(*s, b.String())
}
