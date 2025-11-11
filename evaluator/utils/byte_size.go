package utils

import "fmt"

type ByteSize struct {
	B uint64 // bytes
}

const (
	B  uint64 = 1
	KB uint64 = B << 10 // 1024
	MB uint64 = KB << 10
	GB uint64 = MB << 10
)

func (b ByteSize) MarshalText() ([]byte, error) {
	var bytes []byte
	size := b.B

	switch {
	case size >= GB:
		bytes = fmt.Appendf(bytes, "%.2fGB", float64(size)/float64(GB))
	case size >= MB:
		bytes = fmt.Appendf(bytes, "%.2fMB", float64(size)/float64(MB))
	case size >= KB:
		bytes = fmt.Appendf(bytes, "%.2fKB", float64(size)/float64(KB))
	default:
		bytes = fmt.Appendf(bytes, "%.2fB", float64(size)/float64(B))
	}
	return bytes, nil
}
