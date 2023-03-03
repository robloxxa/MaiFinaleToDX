package jvs

import (
	"fmt"
	"github.com/go-vgo/robotgo"
	"go.bug.st/serial"
	"io"
	"log"
	"time"
)

type JVS struct {
	Port        serial.Port
	Initialized bool

	writeBuf []byte
	readBuf  []byte
	dataBuf  [256]byte
}

func NewFinaleJVS(portName string, mode *serial.Mode) (*JVS, error) {
	port, err := serial.Open(portName, mode)
	if err != nil {
		return nil, err
	}
	return &JVS{
		Port:     port,
		readBuf:  make([]byte, 1),
		writeBuf: make([]byte, 1),
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
		buf, _ := j.Cmd(board, []byte{CMD_READ_DIGITAL, 0x02, 0x02})
		for _, v := range buf {
			fmt.Printf("%08b ", v)
		}
		fmt.Print("\n")
		//readSwitches(buf)
	}
}

func readSwitches(buf []byte) {
	if buf[0]&128 == 128 {
		robotgo.KeyDown(P2_START)
	} else {
		robotgo.KeyUp(P2_START)
	}

	for i, b := range buf[2:6] {
		for k, v := range ButtonInputs[i] {
			if b&k == k {
				robotgo.KeyUp(v)
			} else {
				robotgo.KeyDown(v)
			}
		}
	}
}

func (j *JVS) reset() {
	j.WritePacket(BROADCAST, []byte{CMD_RESET, CMD_RESET_ARG}, 2)
}

func (j *JVS) Cmd(dest byte, data []byte) ([]byte, uint8) {
	var counter uint8
	j.WritePacket(dest, data, uint8(len(data)))

	for {
		// TODO: We could possibly stuck here, need some testing
		if j.ReadByte() != SYNC {
			continue
		}

		if j.ReadByte() != 00 {
			// Not for us, continuing
			continue
		}
		break
	}

	size := j.ReadByte()
	status := j.ReadByte()

	if status == 0x01 {
		log.Fatal("got a wrong status:", status)
	}

	for counter < size {
		b := j.ReadByte()

		if b == MARK {
			b = j.ReadByte() + 1
		}

		j.dataBuf[counter] = b
		counter++
	}

	return j.dataBuf[:counter], status
}

//func (j *JVS) Cmd(dest byte, data []byte) {
//	j.WritePacket(dest, data, uint8(len(data)))
//
//	for {
//		_, err := io.ReadFull(j.Port, syncBuf)
//		if err != nil {
//			log.Println(err)
//			return
//		}
//		if syncBuf[0] != SYNC {
//			fmt.Println("Not sync")
//			continue
//		}
//		_, err = io.ReadFull(j.Port, syncBuf)
//		if err != nil {
//			log.Println(err)
//			return
//		}
//		if syncBuf[0] != 00 {
//			fmt.Println("not 00")
//			continue
//		}
//		break
//	}
//	_, err := io.ReadFull(j.Port, infoBuf)
//	if err != nil {
//		log.Println(err)
//		return
//	}
//	fmt.Printf("Dest %X. Size: %d. Status: %d. Data: ", dest, infoBuf[0], infoBuf[1])
//	n, err := io.ReadFull(j.Port, dataBuf[:infoBuf[0]-1])
//	if err != nil {
//
//		log.Println(err)
//		return
//	}
//	for _, v := range dataBuf[:n] {
//		fmt.Printf("%X ", v)
//	}
//	fmt.Print("\n")
//}

func (j *JVS) WritePacket(dest byte, data []byte, size uint8) {

	j.WriteByte(dest)
	j.WriteByte(size + 1)

	wI := 3
	sum := dest + size + 1

	for i := uint8(0); i < size; i++ {
		if data[i] == SYNC || data[i] == MARK {
			j.WriteByte(MARK)
			j.WriteByte(data[i] - 1)
		} else {
			j.WriteByte(data[i])
		}
		wI++
		sum = uint8(int(sum+data[i]) % 256)
	}
	j.WriteByte(sum)
	//fmt.Print("SENT: ")
	//for i := range writeBuf[:wI] {
	//	fmt.Printf("%X ", writeBuf[i])
	//}
	//fmt.Print("\n")

}

func (j *JVS) ReadByte() byte {
	_, err := io.ReadFull(j.Port, j.readBuf)
	if err != nil {
		log.Fatal(err)
	}
	return j.readBuf[0]
}

func (j *JVS) WriteByte(b byte) {
	j.writeBuf[0] = b
	_, err := j.Port.Write(j.writeBuf)
	if err != nil {
		log.Fatal(err)
	}
}
