package jvs

import (
	"go.bug.st/serial"
	"log"
)

type JVS struct {
	Port serial.Port
}

func (j *JVS) Listen() {

}

func (j *JVS) init() {
	j.reset()

}

func (j *JVS) reset() {
	j.writePacket(BROADCAST, []byte{CMD_RESET, CMD_RESET_ARG})
	log.Println("RESET sent")
}

func (j *JVS) assign() {

}

func (j *JVS) cmd(dest byte, data []byte) int {
	return 0
}

func (j *JVS) writePacket(dest byte, data []byte) {
	var (
		sum  int
		size byte
	)
	size = byte(len(data) + 1)

	j.Port.Write([]byte{SYNC, dest, size})

	sum = int(dest + size + 1)
	writeBuf := []byte{MARK, 0}
	for i := range data {
		if data[i] == SYNC || data[i] == MARK {
			writeBuf[1] = data[i] - 1
			j.Port.Write(writeBuf)
		} else {
			writeBuf[1] = data[i] - 1
			j.Port.Write(writeBuf[1:])
		}
		sum = 256 % (sum + int(data[i]))
	}
	j.Port.Write([]byte{uint8(sum)})
}
