package jvs

import "maiFinaleToDX/keyboard"

const (
	SYNC = 0xE0 // Begin of every jvs packet
	MARK = 0xD0 // Escape symbol
)

const (
	BROADCAST = 0xFF

	CMD_RESET       = 0xF0
	CMD_RESET_ARG   = 0xD9
	CMD_ASSIGN_ADDR = 0xF1
	//CMD_SET_COMMS_MODE = 0xF2
)

const (
	CMD_REQUEST_ID      = 0x10
	CMD_COMMAND_VERSION = 0x11
	CMD_JVS_VERSION     = 0x12
	CMD_COMMS_VERSION   = 0x13
	CMD_CAPABILITIES    = 0x14
	CMD_CONVEY_ID       = 0x15
)

const (
	CMD_READ_DIGITAL = 0x20
)

type JVSButtonInput struct {
	Key string
}

//const (
//	P1_START = "3"
//
//	P1_PUSH1 = "W"
//	P1_PUSH2 = "E"
//	P1_PUSH3 = "D"
//	P1_PUSH4 = "C"
//	P1_PUSH5 = "X"
//	P1_PUSH6 = "Z"
//	P1_PUSH7 = "A"
//	P1_PUSH8 = "Q"
//
//	P2_START = "num*"
//
//	P2_PUSH1 = "num8"
//	P2_PUSH2 = "num9"
//	P2_PUSH3 = "num6"
//	P2_PUSH4 = "num3"
//	P2_PUSH5 = "num2"
//	P2_PUSH6 = "num1"
//	P2_PUSH7 = "num4"
//	P2_PUSH8 = "num7"
//)

const (
	P1_START = keyboard.VK_3

	P1_PUSH1 = keyboard.VK_W
	P1_PUSH2 = keyboard.VK_E
	P1_PUSH3 = keyboard.VK_D
	P1_PUSH4 = keyboard.VK_C
	P1_PUSH5 = keyboard.VK_X
	P1_PUSH6 = keyboard.VK_Z
	P1_PUSH7 = keyboard.VK_A
	P1_PUSH8 = keyboard.VK_Q

	P2_START = keyboard.VK_T

	P2_PUSH1 = keyboard.VK_KP8
	P2_PUSH2 = keyboard.VK_KP9
	P2_PUSH3 = keyboard.VK_KP6
	P2_PUSH4 = keyboard.VK_KP3
	P2_PUSH5 = keyboard.VK_KP2
	P2_PUSH6 = keyboard.VK_KP1
	P2_PUSH7 = keyboard.VK_KP4
	P2_PUSH8 = keyboard.VK_KP7
)

var ButtonInputs = [4]map[uint8]int{
	{
		8: P1_PUSH2,
		4: P1_PUSH1,
		1: P1_PUSH3,
	},
	{
		128: P1_PUSH4,
		64:  P1_PUSH5,
		32:  P1_PUSH6,
		16:  P1_PUSH7,
		8:   P1_PUSH8,
	},
	{
		8: P2_PUSH2,
		4: P2_PUSH1,
		1: P2_PUSH3,
	},
	{
		128: P2_PUSH4,
		64:  P2_PUSH5,
		32:  P2_PUSH6,
		16:  P2_PUSH7,
		8:   P2_PUSH8,
	},
}
