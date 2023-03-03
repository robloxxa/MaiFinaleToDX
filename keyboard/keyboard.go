package keyboard

import (
	"log"
	"syscall"
)

var (
	dll       = syscall.NewLazyDLL("user32.dll")
	procKeyBd = dll.NewProc("keybd_event")
)

var pressedButtons = map[int]bool{}

func KeyDown(key int) {
	if k := pressedButtons[key]; !k {

		keybd_event(key, 0)
		pressedButtons[key] = true
	}
}

func KeyUp(key int) {
	if k := pressedButtons[key]; k {
		keybd_event(key, KEYEVENTF_KEYUP)
		pressedButtons[key] = false
	}
}

func keybd_event(key int, flag int) {
	flag |= KEYEVENTF_SCANCODE
	_, _, err := procKeyBd.Call(uintptr(key), uintptr(key+0x80), uintptr(flag), 0)
	if err != syscall.Errno(0) {
		log.Println(err)
	}
}
