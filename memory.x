/* Device specific memory layout */

/* This file is used to build the cortex-m-rt examples,
   but not other applications using cortex-m-rt. */

MEMORY
{
  m_interrupts          (RX)  : ORIGIN = 0x00000000, LENGTH = 0x00000400
  m_text                (RX)  : ORIGIN = 0x00000400, LENGTH = 0x0001FC00
  m_data                (RW)  : ORIGIN = 0x20000000, LENGTH = 0x00020000
  m_data2               (RW)  : ORIGIN = 0x20200000, LENGTH = 0x000C0000
}

/* The location of the stack can be overridden using the `_stack_start` symbol.
   By default it will be placed at the end of the RAM region */
/* _stack_start = ORIGIN(CCRAM) + LENGTH(CCRAM); */

/* The location of the .text section can be overridden using the `_stext` symbol.
   By default it will place after .vector_table */
/* _stext = ORIGIN(FLASH) + 0x40c; */
