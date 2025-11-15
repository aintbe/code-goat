package constants

type Language string

const (
	C      Language = "c"
	Cpp    Language = "cpp"
	Python Language = "python3"
	Java   Language = "java"
)

type ScmpPolicy uint8

const (
	ScmpUnsafe ScmpPolicy = 0 + iota
	ScmpStrict
	ScmpPython
)
