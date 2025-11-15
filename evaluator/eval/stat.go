package eval

import (
	"time"

	"github.com/aintbe/code-goat/evaluator/constants"
	"github.com/aintbe/code-goat/evaluator/types"
	"github.com/aintbe/code-goat/evaluator/utils"
	"gopkg.in/yaml.v3"
)

type stat[T statType] struct {
	sum T
	Avg T
	Max T
	Min T
}

type statType interface {
	types.ByteSize | time.Duration
}

func (s *stat[T]) Update(value any) {
	switch v := value.(type) {
	case uint64:
		s := any(s).(*stat[types.ByteSize])
		s.sum.B += v
		if s.Max.B < v {
			s.Max.B = v
		}
		if s.Min.B > v || s.Min.B == 0 {
			s.Min.B = v
		}

	case uint32:
		s := any(s).(*stat[time.Duration])
		updateDuration(s, time.Duration(v*constants.MS_TO_NS))

	case time.Duration:
		s := any(s).(*stat[time.Duration])
		updateDuration(s, v)
	}
}

func updateDuration(s *stat[time.Duration], value time.Duration) {
	s.sum += value
	if s.Max < value {
		s.Max = value
	}
	if s.Min > value || s.Min == 0 {
		s.Min = value
	}
}

func (s *stat[T]) Finalize(count int) *stat[T] {
	switch sum := any(s.sum).(type) {
	case types.ByteSize:
		avg := sum.B / uint64(count)
		s.Avg = any(types.ByteSize{B: avg}).(T)

	case time.Duration:
		s.Avg = any(sum / time.Duration(count)).(T)
	}

	return s
}

func (s *stat[T]) MarshalYAML() (interface{}, error) {
	serializable := utils.AutoSerializable{
		"avg": s.Avg,
		"max": s.Max,
		"min": s.Min,
	}
	var node yaml.Node
	if err := node.Encode(serializable); err != nil {
		return nil, err
	}

	// Force inline style to this struct.
	node.Style = yaml.FlowStyle
	return &node, nil
}
