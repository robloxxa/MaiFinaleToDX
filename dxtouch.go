package main

import (
	"go.bug.st/serial"
	"io"
	"log"
)

type DXTouch struct {
	serial.Port
}

func (t *DXTouch) Recv(fe *FinaleTouch) {
	buf := make([]byte, 6)

	for {
		_, err := io.ReadFull(t, buf)
		if err != nil {
			log.Fatal(err)
		}
		log.Printf("%q %v\n", buf, buf[1:5])

		switch buf[3] {
		case CMD_HALT:
			fe.Write(HALT)
		case CMD_STAT:
			fe.Write(STAT)
		case CMD_DX_RSET:

		case CMD_DX_Ratio:
			buf[0] = '('
			buf[5] = ')'
			_, _ = t.Write(buf)
		case CMD_DX_Sens:
			buf[0] = '('
			buf[5] = ')'
			_, _ = t.Write(buf)
		}
	}
}
