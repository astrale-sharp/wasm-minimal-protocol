package main

import (
	"unsafe"

	cbor "github.com/fxamacker/cbor/v2"
)

func main() {}

//go:export hello
func hello() int32 {
	// send output
	sendResultToHost([]byte("Hello from wasm!!!"))
	return 0
}

//go:export double_it
func doubleIt(argLen int32) int32 {
	// get input ("arg")
	input := make([]byte, argLen)
	writeArgsToBuffer(input)
	arg := input
	// send output ("argarg")
	output := cat(arg, arg)
	sendResultToHost(output)
	return 0
}

//go:export concatenate
func concatenate(arg1Len, arg2Len int32) int32 {
	// get input ("arg1", "arg2")
	total := arg1Len + arg2Len
	input := make([]byte, total)
	writeArgsToBuffer(input)
	arg1Start := int32(0)
	arg1End := arg1Start + arg1Len
	arg2Start := arg1End
	arg2End := arg2Start + arg2Len
	arg1 := input[arg1Start:arg1End:arg1End]
	arg2 := input[arg2Start:arg2End:arg2End]
	// send output ("arg1*arg2")
	output := cat(arg1, '*', arg2)
	sendResultToHost(output)
	return 0
}

//go:export shuffle
func shuffle(arg1Len, arg2Len, arg3Len int32) int32 {
	// get input ("arg1", "arg2", "arg3")
	total := arg1Len + arg2Len + arg3Len
	input := make([]byte, total)
	writeArgsToBuffer(input)
	arg1Start := int32(0)
	arg1End := arg1Start + arg1Len
	arg2Start := arg1End
	arg2End := arg2Start + arg2Len
	arg3Start := arg2End
	arg3End := arg3Start + arg3Len
	arg1 := input[arg1Start:arg1End:arg1End]
	arg2 := input[arg2Start:arg2End:arg2End]
	arg3 := input[arg3Start:arg3End:arg3End]
	// send output ("arg3-arg1-arg2")
	output := cat(arg3, '-', arg1, '-', arg2)
	sendResultToHost(output)
	return 0
}

//go:export returns_ok
func returnsOk() int32 {
	sendResultToHost([]byte("This is an `Ok`"))
	return 0
}

//go:export will_panic
func willPanic() int32 {
	panic("DON'T PANIC!")
}

//go:export returns_err
func returnsErr() int32 {
	sendResultToHost([]byte("This is an `Err`"))
	return 1
}

//go:export set_to_a
func setToA(argLen int32) int32 {
	// get input ("arg")
	input := make([]byte, argLen)
	writeArgsToBuffer(input)
	arg := input
	// send output ("aaa")
	for i := range arg {
		arg[i] = 'a'
	}
	sendResultToHost(arg)
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
	writeArgsToBuffer(input)
	var data ComplexDataArgs
	if err := cbor.Unmarshal(input, &data); err != nil {
		sendResultToHost([]byte(err.Error()))
		return 1
	}
	// send output ({float64})
	sum := float64(data.X) + data.Y
	output, err := cbor.Marshal(sum)
	if err != nil {
		sendResultToHost([]byte(err.Error()))
		return 1
	}
	sendResultToHost([]byte(output))
	return 0
}

// --- [ helper functions ] ----------------------------------------------------

func cat(args ...any) []byte {
	var output []byte
	for _, arg := range args {
		switch arg := arg.(type) {
		case byte:
			output = append(output, arg)
		case rune:
			output = append(output, byte(arg))
		case []byte:
			output = append(output, arg...)
		default:
			panic("unsupported type")
		}
	}
	return output
}

// ### [ wasm minimal protocol ] ###############################################

//go:wasmimport typst_env wasm_minimal_protocol_send_result_to_host
func send_result_to_host(ptr, size int32)

func sendResultToHost(buf []byte) {
	ptr := int32(uintptr(unsafe.Pointer(unsafe.SliceData(buf))))
	size := int32(len(buf))
	send_result_to_host(ptr, size)
}

//go:wasmimport typst_env wasm_minimal_protocol_write_args_to_buffer
func write_args_to_buffer(ptr int32)

func writeArgsToBuffer(buf []byte) {
	ptr := int32(uintptr(unsafe.Pointer(unsafe.SliceData(buf))))
	write_args_to_buffer(ptr)
}
