package main

import (
	"fmt"
	"go.bug.st/serial"
	"log"
	"time"
)

func main() {
	dummyPort, err := serial.Open("COM23", &serial.Mode{
		BaudRate: 9600,
		DataBits: 8,
	})
	if err != nil {
		log.Fatal(err)
	}
	//time.AfterFunc(1*time.Millisecond, func() {
	//	dummyPort.Close()
	//})
	writeBuf := make([]byte, 14)
	writeBuf[0] = '('
	writeBuf[11] = 64
	writeBuf[12] = 64
	writeBuf[13] = ')'
	canWrite := true
	canWriteChan := make(chan bool)
	go Listen(dummyPort, canWriteChan)
	for {
		select {
		case c := <-canWriteChan:
			canWrite = c
		default:
			if canWrite {
				SendInput(dummyPort, writeBuf)
			}
		}
	}

}

func Listen(port serial.Port, canWrite chan bool) {
	buf := make([]byte, 6)
	for {
		Recv(port, buf, canWrite)
	}
}

func Recv(port serial.Port, buf []byte, canWrite chan bool) {
	n, err := port.Read(buf)
	if err != nil {
		time.Sleep(500 * time.Millisecond)
		return
	}

	if n == 0 || n > 0 && buf[0] != '{' {
		return
	}

	for n < 6 {
		n2, err := port.Read(buf[n:])
		if err != nil {
			log.Fatal(err)
		}
		n += n2
		fmt.Println(n)
	}
	fmt.Println("sended", buf)
	switch buf[3] {
	case 'L':
		canWrite <- false
	case 'A':
		canWrite <- true
	}
}

func SendInput(port serial.Port, buf []byte) {

	for i := 1; i < 11; i++ {
		if i > 4 && i < 7 {
			buf[i] = 64
		} else {
			buf[i] = 16
		}
	}
	_, _ = port.Write(buf)
	time.Sleep(100 * time.Millisecond)
}
