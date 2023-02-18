package main

import "golang.org/x/sys/windows"

type (
	POINTER_INPUT_TYPE         int32
	POINTER_FLAGS              int32
	POINTER_BUTTON_CHANGE_TYPE int32
)

const (
	POINTER_FLAG_NONE POINTER_FLAGS = 0x00000000
	POINTER_FLAG_NEW                = 0x00000001 << iota
	POINTER_FLAG_INRANGE
	POINTER_FLAG_INCONTACT
	POINTER_FLAG_FIRSTBUTTON
	POINTER_FLAG_SECONDBUTTON
	POINTER_FLAG_THIRDBUTTON
	POINTER_FLAG_FOURTHBUTTON
	POINTER_FLAG_FIFTHBUTTON
	POINTER_FLAG_PRIMARY = 16 << iota
	POINTER_FLAG_CONFIDENCE
	POINTER_FLAG_CANCELED
	POINTER_FLAG_DOWN
	POINTER_FLAG_UPDATE
	POINTER_FLAG_UP
	POINTER_FLAG_WHEEL
	POINTER_FLAG_HWHEEL
	POINTER_FLAG_CAPTURECHANGED
	POINTER_FLAG_HASTRANSFORM
)

const (
	POINTER_CHANGE_NONE             POINTER_BUTTON_CHANGE_TYPE = 0
	POINTER_CHANGE_FIRSTBUTTON_DOWN                            = iota
	POINTER_CHANGE_FIRSTBUTTON_UP
)

const (
	PT_TOUCH POINTER_INPUT_TYPE = 2
)

type POINT struct {
	X, Y int32
}

type POINTER_INFO struct {
	pointerType        POINTER_INPUT_TYPE
	pointerId          uint32
	frameId            uint32
	pointerFlags       POINTER_FLAGS
	sourceDevice       windows.Handle
	hwndTarget         windows.HWND
	ptPixelLocation    POINT
	ptHimetricLocation POINT
	ptPixelLocationRaw POINT
	ptPixelHimetricRaw POINT
	dwTime             uint32
	historyCount       uint32
	InputData          int32
	dwKeyStates        uint32
	PerformanceCount   uint64
	ButtonChangeType   POINTER_BUTTON_CHANGE_TYPE
}
