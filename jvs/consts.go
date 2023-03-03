package jvs

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
	CMD_READ_COIN    = 0x21
	CMD_READ_ANALOG  = 0x22
)
