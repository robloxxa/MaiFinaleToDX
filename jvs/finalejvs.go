package jvs

import (
	"fmt"
	"go.bug.st/serial"
	"io"
	"log"
	"time"
)

type JVS struct {
	Port        serial.Port
	Initialized bool
}

func NewFinaleJVS(portName string, mode *serial.Mode) (*JVS, error) {
	port, err := serial.Open(portName, mode)
	if err != nil {
		return nil, err
	}
	return &JVS{
		Port: port,
	}, nil
}

func (j *JVS) Listen(board uint8) {
	j.reset()
	j.reset()
	time.Sleep(1 * time.Second)
	j.Cmd(BROADCAST, []byte{CMD_ASSIGN_ADDR, board})
	j.Cmd(board, []byte{CMD_REQUEST_ID})
	j.Cmd(board, []byte{CMD_COMMAND_VERSION})
	j.Cmd(board, []byte{CMD_JVS_VERSION})
	j.Cmd(board, []byte{CMD_COMMS_VERSION})
	j.Cmd(board, []byte{CMD_CAPABILITIES})

	for {
		j.Cmd(board, []byte{CMD_READ_DIGITAL, 0x02, 0x02})
	}
}

func (j *JVS) reset() {
	j.Write(BROADCAST, []byte{CMD_RESET, CMD_RESET_ARG}, 2)
}

var (
	syncBuf = make([]byte, 1)
	infoBuf = make([]byte, 2)
	dataBuf = make([]byte, 512)
)

func (j *JVS) Cmd(dest byte, data []byte) {
	j.Write(dest, data, uint8(len(data)))

	for {
		_, err := io.ReadFull(j.Port, syncBuf)
		if err != nil {
			log.Println(err)
			return
		}
		if syncBuf[0] != SYNC {
			fmt.Println("Not sync")
			continue
		}
		_, err = io.ReadFull(j.Port, syncBuf)
		if err != nil {
			log.Println(err)
			return
		}
		if syncBuf[0] != 00 {
			fmt.Println("not 00")
			continue
		}
		break
	}
	_, err := io.ReadFull(j.Port, infoBuf)
	if err != nil {
		log.Println(err)
		return
	}
	fmt.Printf("Dest %X. Size: %d. Status: %d. Data: ", dest, infoBuf[0], infoBuf[1])
	n, err := io.ReadFull(j.Port, dataBuf[:infoBuf[0]-1])
	if err != nil {

		log.Println(err)
		return
	}
	for _, v := range dataBuf[:n] {
		fmt.Printf("%X ", v)
	}
	fmt.Print("\n")
}

var writeBuf = make([]byte, 512)

func (j *JVS) Write(dest byte, data []byte, size uint8) {
	writeBuf[0] = SYNC
	writeBuf[1] = dest
	writeBuf[2] = size + 1
	wI := 3
	sum := dest + size + 1
	for i := uint8(0); i < size; i++ {
		if data[i] == SYNC || data[i] == MARK {
			writeBuf[wI] = MARK
			writeBuf[wI+1] = data[i] - 1
			wI += 2
		} else {
			writeBuf[wI] = data[i]
			wI++
		}
		sum = uint8(int(sum+data[i]) % 256)
	}
	writeBuf[wI] = sum
	wI++
	//fmt.Print("SENT: ")
	//for i := range writeBuf[:wI] {
	//	fmt.Printf("%X ", writeBuf[i])
	//}
	//fmt.Print("\n")
	_, err := j.Port.Write(writeBuf[:wI])
	if err != nil {
		log.Fatal(err)
	}

}
