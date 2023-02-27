package main

import (
	"go.bug.st/serial"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"
)

var (
	DXP1COM   = "COM6"
	DXP2COM   = "COM8"
	FinaleCOM = "COM23" // This depends on what port did you change in windows device manager

	SerialMode = &serial.Mode{BaudRate: 9600, DataBits: 8}
)

func main() {
	var err error
	log.Println("Starting all com port listeners...")
	FETouchSerial, err := NewFinaleTouch(FinaleCOM, SerialMode)
	if err != nil {
		log.Fatal(FinaleCOM, err)
	}

	DXP1TouchSerial, err := NewDXTouch(DXP1COM, SerialMode, 1)
	if err != nil {
		log.Fatal(DXP1COM, err)
	}

	DXP2TouchSerial, err := NewDXTouch(DXP2COM, SerialMode, 2)
	if err != nil {
		log.Fatal(DXP2COM, err)
	}
	cmdChan := make(chan CMDInfo)
	go FETouchSerial.Listen(DXP1TouchSerial, DXP2TouchSerial, cmdChan)
	go DXP1TouchSerial.Listen(cmdChan)
	go DXP2TouchSerial.Listen(cmdChan)

	log.Println("Done! Good luck touchin'")
	s := make(chan os.Signal, 1)
	signal.Notify(s, syscall.SIGINT, syscall.SIGTERM, syscall.SIGHUP)
	<-s
	log.Println("shutting down")
	_, _ = FETouchSerial.Port.Write(HALT)
	time.Sleep(1 * time.Millisecond)
}
