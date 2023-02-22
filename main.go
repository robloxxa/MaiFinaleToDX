package main

import (
	"bufio"
	"fmt"
	"github.com/tevino/abool"
	"go.bug.st/serial"
	"io"
	"log"
	"os"
	"os/signal"
	"sync"
	"syscall"
)

var (
	SerialMode = &serial.Mode{BaudRate: 9600, DataBits: 8}

	DXP1TouchSerial *DXTouch
	DXP2TouchSerial *DXTouch
	FETouchSerial   *FinaleTouch
)

type TestValue struct {
	sync.Mutex
	V []byte
}

func main() {
	mode := &serial.Mode{
		BaudRate: 9600,
		DataBits: 8,
		Parity:   0,
		StopBits: serial.OneStopBit,
	}

	dxPort, err := serial.Open("COM8", mode)
	if err != nil {
		log.Fatal(err)
	}
	testValue := TestValue{
		V: make([]byte, 7),
	}
	fmt.Println(testValue.V)
	testPort, err := serial.Open("COM61", mode)
	if err != nil {
		log.Fatal(err)
	}
	bufioDXPort := bufio.NewWriter(dxPort)

	cond := abool.New()

	go func() {
		buf := make([]byte, 6)

		for {
			_, err := io.ReadFull(dxPort, buf)
			if err != nil {
				log.Fatal(err)
			}
			log.Printf("%q %v\n", buf, buf[1:5])

			switch buf[3] {
			case CMD_HALT:
				cond.UnSet()
			case CMD_STAT:

				cond.Set()
			case CMD_DX_RSET:

			case CMD_DX_Ratio:
				buf[0] = '('
				buf[5] = ')'
				for i := range buf {
					_, _ = dxPort.Write(buf[i : i+1])
				}
			case CMD_DX_Sens:

				buf[0] = '('
				buf[5] = ')'
				for i := range buf {
					_, _ = dxPort.Write(buf[i : i+1])
				}
			}
		}
	}()

	go func() {
		buf := make([]byte, 7)
		for {
			n, err := io.ReadFull(testPort, buf)
			if err != nil {
				log.Fatal(err)
			}
			testValue.Lock()
			testValue.V = buf[:n]
			fmt.Println(testValue.V)
			testValue.Unlock()
		}
	}()

	go func() {
		for {
			if cond.IsSet() {
				bufioDXPort.WriteString("(")
				testValue.Lock()
				_, err := bufioDXPort.Write(testValue.V)
				if err != nil {
					log.Fatal(err)
				}
				testValue.Unlock()
				bufioDXPort.WriteString(")")
			}
		}
	}()
	//
	s := make(chan os.Signal, 1)
	signal.Notify(s, syscall.SIGINT, syscall.SIGTERM, syscall.SIGHUP)
	<-s
	log.Println("shutting down")
	//_, err = dxPort.Write(HALT)
	//if err != nil {
	//	log.Fatal(err)
	//}
	//touches, err := InitializeTouches(1, TOUCH_FEEDBACK_DEFAULT)
	//if err != nil {
	//	log.Fatal("initialize", err)
	//}
	//fmt.Println(touches)
	//ok, err := InjectTouchInput(uint32(len(touches)), touches)
	//if err != nil {
	//	log.Fatal(err)
	//}
	//fmt.Println(ok)
}
