package main

import (
	"go.bug.st/serial"
	"io"
	"log"
)

type FinaleTouch struct {
	serial.Port
}

func (t *FinaleTouch) Recv(buf []byte) {
	n, err := io.ReadAtLeast(t, buf, 6)
	if err != nil {
		log.Fatal(err) // TODO: We shouldn't panic anywhere
	}

	if buf[n-1] == ')' {
		switch buf[3] {
		case CMD_FE_Threshold_Get:

		case CMD_FE_Threshold_Set:
		}
		return
	}

	n, err = io.ReadAtLeast(t, buf[n:], 8)

	// P1
	for i := 1; i < 5; i++ {

	}

	// P2
	for i := 7; i < 11; i++ {

	}
}
