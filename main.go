package main

import (
	"fmt"
	"go.bug.st/serial"
	"io"
	"log"
	"os"
	"os/signal"
	"syscall"
)

var (
	HALT          = []byte("{HALT}") // Stop sensor
	STAT          = []byte("{STAT}") // Start sensor
	LEFT_BRACKET  = []byte("(")
	RIGHT_BRACKET = []byte(")")
)

const (
	A1 = 1 << iota
	B1
	A2
	B2
	C = 10
)

func main() {
	mode := &serial.Mode{
		BaudRate: 9600,
		DataBits: 8,
		Parity:   0,
		StopBits: 0,
	}
	port, err := serial.Open("COM3", mode)
	if err != nil {
		log.Fatal(err)
	}
	_, err = port.Write(STAT)
	if err != nil {
		log.Fatal(err)
	}

	buf := make([]byte, 14)
	//_, err = port.Read(buf)
	//if err != nil {
	//	log.Fatal(err)
	//}
	//err = port.ResetOutputBuffer()
	//if err != nil {
	//	log.Fatal(err)
	//}
	//_, err = io.ReadFull(port, make([]byte, 5))
	go func() {
		for {
			n, err := io.ReadFull(port, buf)
			if err != nil {
				log.Fatal(err)
			}

			fmt.Printf("%q\n", buf[:n])

		}
	}()

	s := make(chan os.Signal, 1)
	signal.Notify(s, syscall.SIGINT, syscall.SIGTERM, syscall.SIGHUP)
	<-s
	log.Println("shutting down")
	port.Write(HALT)

}
