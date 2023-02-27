package jvs

import (
	"fmt"
	"go.bug.st/serial"
	"io"
	"log"
	"time"
)

type JVS struct {
	Port serial.Port
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

func (j *JVS) Listen() {
	j.init()

}

func (j *JVS) init() {
	j.reset()
	j.reset()

	j.cmd(BROADCAST, []byte{CMD_ASSIGN_ADDR, 02})

	//j.cmd(1, []byte{CMD_REQUEST_ID})
	//
	//j.cmd(1, []byte{CMD_COMMAND_VERSION})
	//j.cmd(1, []byte{CMD_JVS_VERSION})
	//j.cmd(1, []byte{CMD_COMMS_VERSION})
	//j.cmd(1, []byte{CMD_CAPABILITIES})
}

func (j *JVS) reset() {
	j.writePacket(BROADCAST, []byte{CMD_RESET, CMD_RESET_ARG})
	time.Sleep(2 * time.Second)
}

func (j *JVS) assign() {

}

var (
	syncBuf = make([]byte, 1)
	infoBuf = make([]byte, 2)
	dataBuf = make([]byte, 512)
)

func (j *JVS) cmd(dest byte, data []byte) {
	j.writePacket(dest, data)

	for {
		_, err := io.ReadFull(j.Port, syncBuf)
		if err != nil {
			log.Fatal(err)
		}
		if syncBuf[0] != SYNC {
			fmt.Println("not sync")
			continue
		}
		break
	}
	for {
		_, err := io.ReadFull(j.Port, syncBuf)
		if err != nil {
			log.Fatal(err)
		}

		break
	}
	n, _ := io.ReadFull(j.Port, infoBuf)
	fmt.Printf("info buf %v\n", infoBuf[:n])
	n, _ = io.ReadFull(j.Port, dataBuf[:infoBuf[0]])
	fmt.Printf("E0 %v%v%v\n", syncBuf, infoBuf, dataBuf[:n])
	time.Sleep(1000 * time.Millisecond)
}

func (j *JVS) writePacket(dest byte, data []byte) {
	var (
		sum  int
		size byte
	)
	size = byte(len(data) + 1)
	fmt.Println(size)
	j.Port.Write([]byte{SYNC, dest, size})

	sum = int(dest) + int(size)
	fmt.Println(sum)
	writeBuf := []byte{MARK, 0}
	for i := range data {
		if data[i] == SYNC || data[i] == MARK {
			writeBuf[1] = data[i] - 1
			j.Port.Write(writeBuf)
		} else {
			writeBuf[1] = data[i]
			j.Port.Write(writeBuf[1:])
		}
		sum = sum + int(data[i])

		fmt.Printf("%X DATA: %d\n", writeBuf[1], data[i])
	}
	j.Port.Write([]byte{uint8(sum % 256)})
}
