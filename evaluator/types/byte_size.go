package types

import (
	"fmt"

	"github.com/aintbe/code-goat/evaluator/constants"
)

type ByteSize struct {
	B uint64 // bytes
}

func (b ByteSize) MarshalText() ([]byte, error) {
	var bytes []byte
	size := b.B

	switch {
	case size >= constants.GiB:
		bytes = fmt.Appendf(bytes, "%.2fGiB", float64(size)/float64(constants.GiB))
	case size >= constants.MiB:
		bytes = fmt.Appendf(bytes, "%.2fMiB", float64(size)/float64(constants.MiB))
	case size >= constants.KiB:
		bytes = fmt.Appendf(bytes, "%.2fKiB", float64(size)/float64(constants.KiB))
	default:
		bytes = fmt.Appendf(bytes, "%.2fB", float64(size)/float64(constants.B))
	}
	return bytes, nil
}
