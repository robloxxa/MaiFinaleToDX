package main

import (
	"flag"
	"go.bug.st/serial"
	"log"
	"maiFinaleToDX/touch"
	"os"
	"os/signal"
	"syscall"
	"time"
)

var (
	TouchDisabled = false
	DXP1COM       = "COM6"
	DXP2COM       = "COM8"
	FinaleCOM     = "COM23" // This depends on what port did you change in Windows device manager

	TouchSerialMode = &serial.Mode{BaudRate: 9600, DataBits: 8}
	JVSSerialMode   = &serial.Mode{BaudRate: 115200, DataBits: 8}
)

func init() {
	flag.StringVar(&DXP1COM, "dxP1TouchPort", "COM6", "Specify Maimai Deluxe Touch Screen COM port for player 1. Default is COM6")
	flag.StringVar(&DXP2COM, "dxP1TouchPort", "COM8", "Specify Maimai Deluxe Touch Screen COM port for player 2. Default is COM8")
	flag.StringVar(&FinaleCOM, "finaleTouchPort", "COM23", "Specify Maimai Finale Touch Screen COM port. Default is COM23")
	flag.BoolVar(&TouchDisabled, "disableTouch", false, "Disable touch screen features")
}

func main() {
	var (
		err           error
		FETouchSerial *touch.FinaleTouch
	)

	flag.Parse()

	if !TouchDisabled {
		log.Println("Starting all com port listeners for TouchScreen...")
		FETouchSerial, err = touch.NewFinaleTouch(FinaleCOM, TouchSerialMode)
		if err != nil {
			log.Fatal(FinaleCOM, err)
		}

		DXP1TouchSerial, err := touch.NewDXTouch(DXP1COM, TouchSerialMode, 1)
		if err != nil {
			log.Fatal(DXP1COM, err)
		}

		DXP2TouchSerial, err := touch.NewDXTouch(DXP2COM, TouchSerialMode, 2)
		if err != nil {
			log.Fatal(DXP2COM, err)
		}
		cmdChan := make(chan touch.CMDInfo)
		go FETouchSerial.Listen(DXP1TouchSerial, DXP2TouchSerial, cmdChan)
		go DXP1TouchSerial.Listen(cmdChan)
		go DXP2TouchSerial.Listen(cmdChan)

		log.Println("Touch Screen initialized, good luck touching")
	}

	s := make(chan os.Signal, 1)
	signal.Notify(s, syscall.SIGINT, syscall.SIGTERM, syscall.SIGHUP)
	<-s
	log.Println("shutting down")
	if FETouchSerial != nil {
		_, _ = FETouchSerial.Port.Write(touch.HALT)
	}
	time.Sleep(1 * time.Millisecond)
}
