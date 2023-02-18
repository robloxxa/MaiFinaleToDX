package main

import (
	"golang.org/x/sys/windows"
	"unsafe"
)

type (
	TOUCH_FEEDBACK uint32
	TOUCH_MASK     int32
)

const ()

const (
	MAX_TOUCH_COUNT int32 = 256

	TOUCH_FEEDBACK_DEFAULT TOUCH_FEEDBACK = iota
	TOUCH_FEEDBACK_INDIRECT
	TOUCH_FEEDBACK_NONE
)

const (
	TOUCH_MASK_CONTACTAREA TOUCH_MASK = 1 << iota
	TOUCH_MASK_ORIENTATION
	TOUCH_MASK_PRESSURE

	TOUCH_FLAG_NONE int32 = 0
)

// POINTER_TOUCH_INFO https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-pointer_touch_info
type POINTER_TOUCH_INFO struct {
	PointerInfo  POINTER_INFO
	TouchFlags   uint32
	TouchMask    TOUCH_MASK
	RcContact    windows.Rect
	RcContactRaw windows.Rect
	Orientation  uint32
	Pressure     uint32
}

var (
	user32DLL                    = windows.NewLazyDLL("user32.dll")
	procInitializeTouchInjection = user32DLL.NewProc("InitializeTouchInjection")
	procInjectTouchInput         = user32DLL.NewProc("InjectTouchInput")
)

func InitializeTouchInjection(maxCount uint32, mode TOUCH_FEEDBACK) (bool, error) {
	success, _, err := procInitializeTouchInjection.Call(
		uintptr(maxCount),
		uintptr(mode),
	)
	ok := *(*bool)(unsafe.Pointer(&success))

	if err != windows.ERROR_SUCCESS {
		return ok, err
	}

	return ok, nil
}

func InjectTouchInput(count uint32, contacts []*POINTER_TOUCH_INFO) (bool, error) {
	success, _, err := procInjectTouchInput.Call(
		uintptr(count),
		uintptr(unsafe.Pointer(&contacts[0])),
	)
	ok := *(*bool)(unsafe.Pointer(&success))
	if err != windows.ERROR_SUCCESS {
		return ok, err
	}
	return ok, nil
}
