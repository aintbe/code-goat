package summary

import (
	"math/big"
	"time"
)

type stat[T statType] struct {
	sum T
	Avg T	`yaml:"avg"`
	Max T	`yaml:"max"`
	Min T	`yaml:"min"`
}

type statType interface {
	primitiveNumber | *big.Int
}

type primitiveNumber interface {
	uint64 | int64 | time.Duration
}

func (s *stat[T]) Update(value T) {
	switch v := any(value).(type) {
	case uint64:
		stat := any(s).(*stat[uint64])
		updatePrimitive(stat, v)
		
	case int64:
		stat := any(s).(*stat[int64])
		updatePrimitive(stat, v)
	
	case time.Duration:
		stat := any(s).(*stat[time.Duration])
		updatePrimitive(stat, v)
		
	case *big.Int:
		stat := any(s).(*stat[*big.Int])
		updateBigInt(stat, v)
	}
}

func updatePrimitive[T primitiveNumber](s *stat[T], value T) *stat[T] {
    s.sum += value
	if s.Max < value {
		s.Max = value
	}
	if  s.Min > value || s.Min == 0 {
		s.Min = value
	}

	return s
}

var ZERO = big.NewInt(0)

func updateBigInt(s *stat[*big.Int], value *big.Int) {
	s.sum.Add(s.sum, value)
	if s.Max.Cmp(value) < 0 {
		s.Max = value
	}
	if s.Min.Cmp(value) > 0 || s.Min.Cmp(ZERO) == 0{
		s.Min = value
	}
}

func (s *stat[T]) Finalize(count int64) *stat[T] {
	switch sum := any(s.sum).(type) {
	case uint64:
		s.Avg = any(sum / uint64(count)).(T)
	
	case int64:
		s.Avg = any(sum / count).(T)

	case time.Duration:
		s.Avg = any(sum / time.Duration(count)).(T)

	case *big.Int:
		avg := any(s.Avg).(*big.Int)
		avg.Div(sum, big.NewInt(count))
	}

	return s
}
