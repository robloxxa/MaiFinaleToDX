package main

import (
	"github.com/tevino/abool/v2"
	"go.bug.st/serial"
	"log"
)

type CMDInfo struct {
	P    *DXTouch
	NumP int
	Data []byte
}

type FinaleTouch struct {
	Port serial.Port

	P1Active bool
	P2Active bool

	DataReceived abool.AtomicBool
}

func NewFinaleTouch(portName string, portMode *serial.Mode) (*FinaleTouch, error) {
	port, err := serial.Open(portName, portMode)
	if err != nil {
		return nil, err
	}
	return &FinaleTouch{Port: port}, nil
}

func (t *FinaleTouch) Listen(p1 *DXTouch, p2 *DXTouch, cmdChan chan CMDInfo) {
	t.DataReceived.UnSet()
	feBuf := []byte{'(', '@', '@', '@', '@', '@', '@', '@', '@', '@', '@', '@', '@', ')'} // Length 14
	dxBuf := []byte{'(', 0, 0, 0, 0, 0, 0, 0, ')'}
	_, _ = t.Port.Write(STAT)
	for {
		select {
		case cmd := <-cmdChan:
			log.Printf("P%d: %q %v\n", cmd.P.Num, cmd.Data, cmd.Data[1:5])
			switch cmd.Data[3] {
			case CMD_HALT:
				cmd.P.Active = false
			case CMD_STAT:
				cmd.P.Active = true
			//case CMD_DX_RSET:
			//TODO: I still don't know what RSET does and how to react to it
			case CMD_DX_Ratio:
				cmd.Data[0] = '('
				cmd.Data[5] = ')'
				cmd.P.Write(cmd.Data)
			case CMD_DX_Sens:
				cmd.Data[0] = '('
				cmd.Data[5] = ')'
				cmd.P.Write(cmd.Data)
			}
		default:
			// This looks very weird (for me at least). But it's the only way to read fast without blocking
			// The only reason I am doing this - minimum serial.Port.SetReadDelay, which is 15 ms (windows limits). Imo it is big
			// Maybe go just isn't the best language for this :]

			n, err := t.Port.Read(feBuf)
			if err != nil {
				log.Fatal(err)
			}

			if n == 0 || n > 0 && feBuf[0] != '(' {
				break
			}

			for n < 6 {
				n2, err := t.Port.Read(feBuf[n:])
				if err != nil {
					log.Println(err)
					break
				}
				n += n2
			}

			log.Printf("FE: %v\n", feBuf)
			if p1.Active {
				convertFEInputToDX(feBuf, dxBuf, p1)
				log.Printf("DXP1: %v\n", dxBuf)
				p1.Write(dxBuf)
			}

			if p2.Active {
				convertFEInputToDX(feBuf, dxBuf, p2)
				log.Printf("DXP2: %v\n", dxBuf)
				p2.Write(dxBuf)
			}

		}
	}
}

// Convert input from
func convertFEInputToDX(feBuffer, dxBuffer []byte, p *DXTouch) {
	var sIndex, eIndex int
	if p.Num == 1 {
		sIndex = 1
		eIndex = 5
	} else if p.Num == 2 {
		sIndex = 7
		eIndex = 11
	} else {
		panic("wrong player num")
	}
	resetDxBuffer(dxBuffer)
	for i, v := range feBuffer[sIndex:eIndex] {
		for k, ar := range FEAreas[i] {
			if v&k != k {
				continue
			}

			if k == 16 {
				// Little workaround for C1/C2 area so players will be able to switch mapping style in dx song selection
				if p.CAreaSwitch {
					dxBuffer[ar.Index] |= ar.Bit
					p.CAreaSwitch = false
				} else {
					dxBuffer[ar.Index] |= ar.Area1.Bit
					p.CAreaSwitch = true
				}
				continue
			}

			dxBuffer[ar.Index] |= ar.Bit
			dxBuffer[ar.Area1.Index] |= ar.Area1.Bit
			dxBuffer[ar.Area2.Index] |= ar.Area2.Bit
		}
	}
}

func resetDxBuffer(buf []byte) {
	for i := 1; i < 8; i++ {
		buf[i] = 0
	}
}
