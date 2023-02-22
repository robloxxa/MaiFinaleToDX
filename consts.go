package main

var (
	HALT = []byte("{HALT}") // Stop sensor
	STAT = []byte("{STAT}") // Start sensor
)

const (
	AREA_1 = 1 << iota
	AREA_2
	AREA_3
	AREA_4
	AREA_5
	AREA_ALL = AREA_1 | AREA_2 | AREA_3 | AREA_4 | AREA_5
)

const (
	CMD_HALT = 'L'
	CMD_STAT = 'A'
)

// Maimai Finale specific values.
// All info about protocols can be found here:
// https://bsnk.me/eamuse/sega/hardware/touch.html
const (
	FE_NULL = '@'

	CMD_FE_Threshold_Get = 't' // { L/R Sensor ->t<- h }
	CMD_FE_Threshold_Set = 'k' // { L/R Sensor ->k<- threshold }
)

// Maimai DX specific values.
// Below is Touch input protocol. x - non used.
// { A1/A2/A3/A4/A5 A6/A7/A8/B1/B2 B3/B4/B5/B6/B7 B8/C1/C2/D1/D2 D3/D4/D5/D6/D7 D8/E1/E2/E3/E4 E5/E6/E7/E8/x }
const (
	DX_NULL = 0x0

	DX_AREA_C = AREA_2 | AREA_3

	CMD_DX_RSET  = 'E' // { R S E T } Not really sure what it does, maybe resets threshold values?
	CMD_DX_Ratio = 'r' // { L/R A/B/C/D/E/F r ? } No equivalent in Finale
	CMD_DX_Sens  = 'k' // {  }Touch sensitivity this is analog of
)
