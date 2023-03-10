package touch

var (
	HALT = []byte("{HALT}") // Stop sensor
	STAT = []byte("{STAT}") // Start sensor
)

const (
	CMD_HALT = 'L'
	CMD_STAT = 'A'
)

// Maimai Finale specific values.
// All info about protocols can be found here:
// https://bsnk.me/eamuse/sega/hardware/touch.html
const (
	CMD_FE_Threshold_Get = 't' // { L/R Sensor ->t<- h }
	CMD_FE_Threshold_Set = 'k' // { L/R Sensor ->k<- threshold }
)

// Maimai DX specific values.
// Below is Touch input protocol. x - non used.
// { A1/A2/A3/A4/A5 A6/A7/A8/B1/B2 B3/B4/B5/B6/B7 B8/C1/C2/D1/D2 D3/D4/D5/D6/D7 D8/E1/E2/E3/E4 E5/E6/E7/E8/x }
const (
	CMD_DX_RSET  = 'E' // { R S E T } Not really sure what it does, maybe resets threshold values?
	CMD_DX_Ratio = 'r' // { L/R A/B/C/D/E/F r ? } No equivalent in Finale
	CMD_DX_Sens  = 'k' // {  } Touch sensitivity this is analog of Threshold in Finale
)

type DXInput struct {
	Index int
	Bit   uint8
}

type FEInput struct {
	DXInput
	Area1 DXInput
	Area2 DXInput
}

var (
	A1 = DXInput{1, 1}
	A2 = DXInput{1, 2}
	A3 = DXInput{1, 4}
	A4 = DXInput{1, 8}
	A5 = DXInput{1, 16}

	A6 = DXInput{2, 1}
	A7 = DXInput{2, 2}
	A8 = DXInput{2, 4}
	B1 = DXInput{2, 8}
	B2 = DXInput{2, 16}

	B3 = DXInput{3, 1}
	B4 = DXInput{3, 2}
	B5 = DXInput{3, 4}
	B6 = DXInput{3, 8}
	B7 = DXInput{3, 16}

	B8 = DXInput{4, 1}
	C1 = DXInput{4, 2}
	C2 = DXInput{4, 4}
	D1 = DXInput{4, 8}
	D2 = DXInput{4, 16}

	D3 = DXInput{5, 1}
	D4 = DXInput{5, 2}
	D5 = DXInput{5, 4}
	D6 = DXInput{5, 8}
	D7 = DXInput{5, 16}

	D8 = DXInput{6, 1}
	E1 = DXInput{6, 2}
	E2 = DXInput{6, 4}
	E3 = DXInput{6, 8}
	E4 = DXInput{6, 16}

	E5 = DXInput{7, 1}
	E6 = DXInput{7, 2}
	E7 = DXInput{7, 4}
	E8 = DXInput{7, 8}
)

var FEAreas = [4]map[uint8]FEInput{
	{
		1: {A1, D1, D2},
		2: {B1, E1, E2}, // B1
		4: {A2, D2, D3}, // A2
		8: {B2, E2, E3}, // B2
	},
	{
		1: {A3, D3, D4},
		2: {B3, E3, E4},
		4: {A4, D4, D5},
		8: {B4, E4, E5},
	},
	{
		1: {A5, D5, D6},
		2: {B5, E5, E6},
		4: {A6, D6, D7},
		8: {B6, E6, E7},
	},
	{
		1:  {A7, D7, D8},
		2:  {B7, E7, E8},
		4:  {A8, D8, D1},
		8:  {B8, E8, E1},
		16: {C1, C2, DXInput{}},
	},
}
