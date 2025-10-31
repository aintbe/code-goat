package config

import (
	"bytes"
	"strconv"
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

func (s *StringSlice) AddUInt(name string, value uint64) {
	if value != 0 {
		s.Concat(name, strconv.FormatUint(value, 10))
	}
}

func (s *StringSlice) AddString(name string, value string) {
	if value != "" {
		s.Concat(name, value)
	}
}
func (s *StringSlice) Extend(name string, values []string) {
	for _, value := range values {
		s.Concat(name, value)
	}
}

func (s *StringSlice) Concat(name string, value string) {
	var b bytes.Buffer
	b.WriteString(name)
	b.WriteString(value)
	*s = append(*s, b.String())
}
