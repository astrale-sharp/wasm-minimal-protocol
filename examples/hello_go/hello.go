package main

import (
	"unsafe"

	cbor "github.com/fxamacker/cbor/v2"
)

// ===
// functions for the protocol

//go:wasmimport typst_env wasm_minimal_protocol_write_args_to_buffer
func write_args_to_buffer(ptr int32)

func WriteArgsToBuffer(argBuf []byte) {
	ptr := int32(uintptr(unsafe.Pointer(unsafe.SliceData(argBuf))))
	write_args_to_buffer(ptr)
}

//go:wasmimport typst_env wasm_minimal_protocol_send_result_to_host
func send_result_to_host(ptr, size int32)

func SendResultToHost(resBuf []byte) {
	size := int32(len(resBuf))
	ptr := int32(uintptr(unsafe.Pointer(unsafe.SliceData(resBuf))))
	send_result_to_host(ptr, size)
}

// ===

func main() {}

//go:export hello
func hello() int32 {
	const msg = "Hello from wasm!!!"
	SendResultToHost([]byte(msg))
	return 0
}

//go:export double_it
func doubleIt(argLen int32) int32 {
	buf := make([]byte, argLen*2)
	WriteArgsToBuffer(buf)

	copy(buf[argLen:], buf[:argLen])
	SendResultToHost(buf)
	return 0
}

//go:export concatenate
func concatenate(arg1Len, arg2Len int32) int32 {
	totalLen := arg1Len + arg2Len + 1
	buf := make([]byte, totalLen)
	WriteArgsToBuffer(buf)

	copy(buf[arg1Len+1:], buf[arg1Len:])
	buf[arg1Len] = '*'
	SendResultToHost(buf)
	return 0
}

//go:export shuffle
func shuffle(arg1Len, arg2Len, arg3Len int32) int32 {
	totalLen := arg1Len + arg2Len + arg3Len
	argBuf := make([]byte, totalLen)
	WriteArgsToBuffer(argBuf)

	arg1 := argBuf[:arg1Len]
	arg2 := argBuf[arg1Len : arg1Len+arg2Len]
	arg3 := argBuf[arg1Len+arg2Len:]

	// Pre-allocate with exact capacity needed
	resBuf := make([]byte, 0, totalLen+2)
	resBuf = append(resBuf, arg3...)
	resBuf = append(resBuf, '-')
	resBuf = append(resBuf, arg1...)
	resBuf = append(resBuf, '-')
	resBuf = append(resBuf, arg2...)

	SendResultToHost(resBuf)
	return 0
}

//go:export returns_ok
func returnsOk() int32 {
	const msg = "This is an `Ok`"
	SendResultToHost([]byte(msg))
	return 0
}

//go:export returns_err
func returnsErr() int32 {
	const msg = "This is an `Err`"
	SendResultToHost([]byte(msg))
	return 1
}

//go:export will_panic
func willPanic() int32 {
	panic("Panicking, this message will not be seen...")
}

//go:export set_to_a
func setToA(argLen int32) int32 {
	// get input ("arg")
	buf := make([]byte, argLen)
	WriteArgsToBuffer(buf)
	// send output ("aaa")
	for i := range buf {
		buf[i] = 'a'
	}
	SendResultToHost(buf)
	return 0
}

type ComplexDataArgs struct {
	X int32
	Y float64
}

//go:export complex_data
func complexData(dataLen int32) int32 {
	// get input ({int32, float64})
	input := make([]byte, dataLen)
	WriteArgsToBuffer(input)
	var data ComplexDataArgs
	if err := cbor.Unmarshal(input, &data); err != nil {
		SendResultToHost([]byte(err.Error()))
		return 1
	}
	// send output ({float64})
	sum := float64(data.X) + data.Y
	output, err := cbor.Marshal(sum)
	if err != nil {
		SendResultToHost([]byte(err.Error()))
		return 1
	}
	SendResultToHost([]byte(output))
	return 0
}
