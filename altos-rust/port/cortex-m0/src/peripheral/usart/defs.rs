// Base addresses for USART 1 and 2
pub const USART1_ADDR: u32 = 0x4001_3800;
pub const USART2_ADDR: u32 = 0x4000_4400;

// ------------------------------------
// USARTx - CR1 Bit definitions
// ------------------------------------
pub const CR1_OFFSET: u32 = 0x0;
pub const CR1_UE:     u32 = 0b1;
pub const CR1_UESM:   u32 = 0b1 << 1;
pub const CR1_RE:     u32 = 0b1 << 2;
pub const CR1_TE:     u32 = 0b1 << 3;
pub const CR1_IDLEIE: u32 = 0b1 << 4;
pub const CR1_RXNEIE: u32 = 0b1 << 5;
pub const CR1_TCIE:   u32 = 0b1 << 6;
pub const CR1_TXEIE:  u32 = 0b1 << 7;
pub const CR1_PEIE:   u32 = 0b1 << 8;
pub const CR1_PS:     u32 = 0b1 << 9;
pub const CR1_PCE:    u32 = 0b1 << 10;
pub const CR1_WAKE:   u32 = 0b1 << 11;
pub const CR1_M0:     u32 = 0b1 << 12;
pub const CR1_MME:    u32 = 0b1 << 13;
pub const CR1_CMIE:   u32 = 0b1 << 14;
pub const CR1_OVER8:  u32 = 0b1 << 15;
// pub const CR1_DEDT: u32 = ??; // this is bits 16-20
// pub const CR1_DEAT: u32 = ??; // this is bits 21-25
pub const CR1_RTOIE:  u32 = 0b1 << 26;
pub const CR1_EOBIE:  u32 = 0b1 << 27;
pub const CR1_M1:     u32 = 0b1 << 28;
// Bits 29 - 31 are reserved and must be kept at reset value.

// ------------------------------------
// USARTx - CR2 bit definitions
// ------------------------------------
pub const CR2_OFFSET:    u32 = 0x0000_0004;
// Bits 0 - 3 are reserved and must be kept at reset value.
pub const CR2_ADDM7:     u32 = 0b1 << 4;
pub const CR2_LBDL:      u32 = 0b1 << 5;
pub const CR2_LBDIE:     u32 = 0b1 << 6;
// Bit 7 is reserved and must be kept at reset value.
pub const CR2_LBCL:      u32 = 0b1 << 8;
pub const CR2_CPHA:      u32 = 0b1 << 9;
pub const CR2_CPOL:      u32 = 0b1 << 10;
pub const CR2_CLKEN:     u32 = 0b1 << 11;
pub const CR2_STOP_BIT0: u32 = 0b1 << 12;
pub const CR2_STOP_BIT1: u32 = 0b1 << 13;
pub const CR2_LINEN:     u32 = 0b1 << 14;
pub const CR2_SWAP:      u32 = 0b1 << 15;
pub const CR2_RXINV:     u32 = 0b1 << 16;
pub const CR2_TXINV:     u32 = 0b1 << 17;
pub const CR2_DATAINV:   u32 = 0b1 << 18;
pub const CR2_MSBFIRST:  u32 = 0b1 << 19;
pub const CR2_ABREN:     u32 = 0b1 << 20;
pub const CR2_ABRMOD0:   u32 = 0b1 << 21;
pub const CR2_ABRMOD1:   u32 = 0b1 << 22;
pub const CR2_RTOEN:     u32 = 0b1 << 23;
pub const CR2_ADD:       u32 = 0b1111 << 24; // This might need to change
pub const CR2_ADD1:      u32 = 0b1111 << 28; // This might need to change

// ------------------------------------
// USARTx - CR3 bit definitions
pub const CR3_OFFSET: u32 = 0x0000_0008;
pub const CR3_RTSE:   u32 = 0b1 << 8;
pub const CR3_CTSE:   u32 = 0b1 << 9;

// ------------------------------------
// USARTx - BRR bit definitions
// ------------------------------------
pub const BRR_OFFSET: u32 = 0x0000_000C;
// Bits 16 - 31 are reserved and must be kept at reset value
//
// BRR[15:4] = USARTDIV[15:4]
//
// When OVER8 = 0
//      BRR[3:0] = USARTDIV[3:0]
// When OVER8 = 1
//      BRR[2:0] = USARTDIV[3:0] shifted 1 bit to the right when OVER
//      BRR[3] must be kept cleared

// Value of 5000 for USARTDIV yields a 9600 Kb/s baud rate
pub const USARTDIV_9600: u32 = 5000;
// Value of 417 for USARTDIV yields roughly a 115200 Kb/s baud rate
pub const USARTDIV_115200: u32 = 416;

// ------------------------------------
// USARTx - GTPR bit definitions
// ------------------------------------
pub const GTPR_OFFSET: u32 = 0x0000_0010;
