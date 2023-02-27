package main

import (
	"go.bug.st/serial"
	"io"
	"log"
)

type DXTouch struct {
	Port        serial.Port
	Num         int
	Active      bool
	CAreaSwitch DXInput
}

func NewDXTouch(portName string, serialMode *serial.Mode, playerNum int) (*DXTouch, error) {
	port, err := serial.Open(portName, serialMode)
	if err != nil {
		return nil, err
	}
	return &DXTouch{Num: playerNum, Port: port}, nil
}

func (t *DXTouch) Listen(cmdChan chan CMDInfo) {
	buf := make([]byte, 6)

	for {
		_, err := io.ReadFull(t.Port, buf)
		if err != nil {
			log.Fatal(err)
		}
		tempBuf := make([]byte, 6)
		copy(tempBuf, buf)
		cmdChan <- CMDInfo{
			P:    t,
			Data: tempBuf,
		}
	}
}

func (t *DXTouch) Write(buf []byte) {
	_, err := t.Port.Write(buf)
	if err != nil {
		t.Active = false
	}
}
