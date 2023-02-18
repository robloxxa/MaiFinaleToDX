package main

import (
	"log"
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
	B = 0x00010000
)

func main() {
	//mode := &serial.Mode{
	//	BaudRate: 9600,
	//	DataBits: 8,
	//	Parity:   0,
	//	StopBits: 0,
	//}
	//port, err := serial.Open("COM3", mode)
	//if err != nil {
	//	log.Fatal(err)
	//}
	//_, err = port.Write(STAT)
	//if err != nil {
	//	log.Fatal(err)
	//}
	//
	//buf := make([]byte, 14)
	//
	//go func() {
	//	for {
	//		n, err := io.ReadFull(port, buf)
	//		if err != nil {
	//			log.Fatal(err)
	//		}
	//
	//		fmt.Printf("%q\n", buf[:n])
	//
	//	}
	//}()
	//
	//s := make(chan os.Signal, 1)
	//signal.Notify(s, syscall.SIGINT, syscall.SIGTERM, syscall.SIGHUP)
	//<-s
	//log.Println("shutting down")
	//_, err = port.Write(HALT)
	//if err != nil {
	//	log.Fatal(err)
	//}
	_, err := InitializeTouchInjection(1, TOUCH_FEEDBACK_DEFAULT)
	if err != nil {
		log.Fatal(err)
	}
	d := new(POINTER_TOUCH_INFO)
	ok, err := InjectTouchInput(1, []*POINTER_TOUCH_INFO{
		d,
	})

	if err != nil {
		log.Fatal(err)
	}
	log.Println(ok)
}
