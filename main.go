package main

import (
	"go.bug.st/serial"
	"log"
	"os"
	"os/signal"
	"syscall"
)

var (
	DXP1COM   = "COM6"
	DXP2COM   = "COM8"
	FinaleCOM = "COM23" // This depends on what port did you change in windows device manager

	SerialMode = &serial.Mode{BaudRate: 9600, DataBits: 8}

	DXP1TouchSerial = &DXTouch{}
	//DXP2TouchSerial = &DXTouch{}
	FETouchSerial = &FinaleTouch{}
)

func main() {
	var err error
	DXP1TouchSerial.Port, err = serial.Open(DXP1COM, SerialMode)
	if err != nil {
		log.Fatal(err) // TODO: make reconnect method
	}
	//DXP2TouchSerial.Port, err = serial.Open(DXP2COM, SerialMode)
	FETouchSerial.Port, err = serial.Open(FinaleCOM, SerialMode)
	if err != nil {
		log.Fatal(err)
	}
	go FETouchSerial.Recv(DXP1TouchSerial)
	go DXP1TouchSerial.Recv(FETouchSerial)
	//go DXP2TouchSerial.Recv(FETouchSerial)

	s := make(chan os.Signal, 1)
	signal.Notify(s, syscall.SIGINT, syscall.SIGTERM, syscall.SIGHUP)
	<-s
	log.Println("shutting down")
	FETouchSerial.Write(HALT)
}
