package main

import (
	"errors"
	"fmt"
	"golang.org/x/sys/windows"
	"unsafe"
)

type (
	TOUCH_FEEDBACK uint32
	TOUCH_MASK     int32
)

const (
	MAX_TOUCH_COUNT int32 = 256

	TOUCH_FEEDBACK_DEFAULT TOUCH_FEEDBACK = iota
	TOUCH_FEEDBACK_INDIRECT
	TOUCH_FEEDBACK_NONE
)

const (
	TOUCH_MASK_CONTACTAREA uint32 = 1 << iota
	TOUCH_MASK_ORIENTATION
	TOUCH_MASK_PRESSURE
	TOUCH_MASK_ALL = TOUCH_MASK_PRESSURE | TOUCH_MASK_ORIENTATION | TOUCH_MASK_CONTACTAREA

	TOUCH_FLAG_NONE uint32 = 0
)

// POINTER_TOUCH_INFO https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-pointer_touch_info
type POINTER_TOUCH_INFO struct {
	pointerInfo  POINTER_INFO
	touchFlags   uint32
	touchMask    uint32
	rcContact    windows.Rect
	rcContactRaw windows.Rect
	orientation  uint32
	pressure     uint32
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

func InjectTouchInput(count uint32, contacts []POINTER_TOUCH_INFO) (bool, error) {
	slice := unsafe.Slice(&contacts[0], count)

	fmt.Println(len(slice))
	success, _, err := procInjectTouchInput.Call(
		uintptr(count),
		uintptr(unsafe.Pointer(&slice)),
	)
	ok := *(*bool)(unsafe.Pointer(&success))
	if err != windows.ERROR_SUCCESS {
		return ok, err
	}
	return ok, nil
}

func InitializeTouches(maxTouches int, dwMode TOUCH_FEEDBACK) ([]POINTER_TOUCH_INFO, error) {
	touches := make([]POINTER_TOUCH_INFO, maxTouches)

	for i := range touches {
		touches[i].pointerInfo = POINTER_INFO{
			pointerType:     PT_TOUCH,
			pointerId:       uint32(i),
			pointerFlags:    POINTER_FLAG_NEW,
			ptPixelLocation: POINT{950, 540},
		}
		touches[i].touchFlags = TOUCH_FLAG_NONE
		touches[i].touchMask = TOUCH_MASK_ALL
		touches[i].rcContact = windows.Rect{
			Left:   touches[i].pointerInfo.ptPixelLocation.X - 5,
			Top:    touches[i].pointerInfo.ptPixelLocation.Y - 5,
			Right:  touches[i].pointerInfo.ptPixelLocation.X + 5,
			Bottom: touches[i].pointerInfo.ptPixelLocation.Y + 5,
		}
		touches[i].orientation = 90
		touches[i].pressure = 32000
	}

	ok, err := InitializeTouchInjection(uint32(maxTouches), dwMode)
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, errors.New("InitializeTouchInjection returned false")
	}
	return touches, nil
}
