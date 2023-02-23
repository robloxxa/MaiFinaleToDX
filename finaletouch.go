package main

import (
	"go.bug.st/serial"
	"io"
	"log"
)

type CMDInfo struct {
	P    *DXTouch
	NumP int
	Data []byte
}

type FinaleTouch struct {
	serial.Port
}

var FEAreas = [4]map[uint8]FEInput{
	{
		1: {A1, D1, D2},
		2: {B1, E1, E2}, // B1
		4: {A2, D2, D3}, // A2
		8: {B2, E2, E3}, // B2
	},
	{
		1: {A3, D3, D4},
		2: {B3, E3, E4},
		4: {A4, D4, D5},
		8: {B4, E4, E5},
	},
	{
		1: {A5, D5, D6},
		2: {B5, E5, E6},
		4: {A6, D6, D7},
		8: {B6, E6, E7},
	},
	{
		1:  {A7, D7, D8},
		2:  {B7, E7, E8},
		4:  {A8, D8, D1},
		8:  {B8, E8, E1},
		16: {C1, C2, DXInput{}},
	},
}

type DXInput struct {
	Index int
	Bit   uint8
}
type FEInput struct {
	DXInput
	Area1 DXInput
	Area2 DXInput
}

func (t *FinaleTouch) Recv(dxP1 *DXTouch) {
	buf := make([]byte, 14)
	p1InputBuf := []byte{'(', 0, 0, 0, 0, 0, 0, 0, ')'}
	for {
		n, err := io.ReadFull(t, buf)
		if err != nil {
			log.Fatal(err) // TODO: We shouldn't panic anywhere
		}
		//if buf[:5][5] == ')' {
		//	switch buf[3] {
		//	case CMD_FE_Threshold_Get:
		//		// TODO:
		//	case CMD_FE_Threshold_Set:
		//		// TODO: figure out threshold
		//	}
		//	return
		//}
		// P1
		resetBuf(p1InputBuf)
		for i, v := range buf[:n][1:5] {
			for k, ar := range FEAreas[i] {
				if v&k != k {
					continue
				}
				p1InputBuf[ar.Index] |= ar.Bit
				p1InputBuf[ar.Area1.Index] |= ar.Area1.Bit
				p1InputBuf[ar.Area2.Index] |= ar.Area2.Bit
			}
		}
		_, err = dxP1.Write(p1InputBuf)
		if err != nil {
			log.Fatal(err)
		}
		//if t.P2Ready {
		//	resetBuf(p2InputBuf)
		//	for i, v := range buf[:n][7:11] {
		//		for k, ar := range FEAreas[i] {
		//			if v&k != k {
		//				continue
		//			}
		//			p2InputBuf[ar.Index] |= ar.Bit
		//			p2InputBuf[ar.Area1.Index] |= ar.Area1.Bit
		//			p2InputBuf[ar.Area2.Index] |= ar.Area2.Bit
		//		}
		//	}
		//	_, err = dxP2.Write(p2InputBuf)
		//	if err != nil {
		//		log.Fatal(err)
		//	}
		//}

	}

}

func resetBuf(buf []byte) {
	buf[1] = 0
	buf[2] = 0
	buf[3] = 0
	buf[4] = 0
	buf[5] = 0
	buf[6] = 0
	buf[7] = 0
}
