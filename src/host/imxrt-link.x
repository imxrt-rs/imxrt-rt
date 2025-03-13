/* ===--- Begin imxrt-link.x ---===
 * This section of the linker script is a fork of the default linker script provided by
 * imxrt-rt, version 0.7.1. It's modified to support the needs of imxrt-rt.
 */

/* # Entry point = reset vector */
EXTERN(__RESET_VECTOR);
EXTERN(Reset);
ENTRY(Reset);

/* # Exception vectors */
/* This is effectively weak aliasing at the linker level */
/* The user can override any of these aliases by defining the corresponding symbol themselves (cf.
   the `exception!` macro) */
EXTERN(__EXCEPTIONS); /* depends on all the these PROVIDED symbols */

EXTERN(DefaultHandler);
EXTERN(__pre_init);

PROVIDE(NonMaskableInt = DefaultHandler);
EXTERN(HardFaultTrampoline);
PROVIDE(MemoryManagement = DefaultHandler);
PROVIDE(BusFault = DefaultHandler);
PROVIDE(UsageFault = DefaultHandler);
PROVIDE(SecureFault = DefaultHandler);
PROVIDE(SVCall = DefaultHandler);
PROVIDE(DebugMonitor = DefaultHandler);
PROVIDE(PendSV = DefaultHandler);
PROVIDE(SysTick = DefaultHandler);

PROVIDE(DefaultHandler = DefaultHandler_);
PROVIDE(HardFault = HardFault_);

/* # Interrupt vectors */
EXTERN(__INTERRUPTS); /* `static` variable similar to `__EXCEPTIONS` */

/* # Sections */
SECTIONS
{
  .stack (NOLOAD) : ALIGN(8)
  {
    __estack = .;
    . += ALIGN(__stack_size, 8);
    __sstack = .;
    /* Symbol expected by cortex-m-rt */
    _stack_start = __sstack;
  } > REGION_STACK

  .vector_table : ALIGN(1024)
  {
    FILL(0xff);
    __vector_table = .;
    __svector_table = .;

    /* Initial Stack Pointer (SP) value */
    LONG(__sstack);

    /* Reset vector */
    KEEP(*(.vector_table.reset_vector)); /* this is the `__RESET_VECTOR` symbol */
    __reset_vector = .;

    /* Exceptions */
    KEEP(*(.vector_table.exceptions)); /* this is the `__EXCEPTIONS` symbol */
    __eexceptions = .;

    /* Device specific interrupts */
    KEEP(*(.vector_table.interrupts)); /* this is the `__INTERRUPTS` symbol */
    __evector_table = .;
  } > REGION_VTABLE AT> REGION_LOAD_VTABLE
  __sivector_table = LOADADDR(.vector_table);

  /* This section guarantees VMA = LMA to allow the execute-in-place entry point to be inside the image. */
  .xip : ALIGN(4)
  {
    /* Included here if not otherwise included in the boot header. */
    *(.Reset);
    *(.__pre_init);
    *(.xip .xip.*);
  } > REGION_LOAD_TEXT

  .text : ALIGN(4)
  {
    FILL(0xff);
    __stext = .;
    *(.text .text.*);
    /* The HardFaultTrampoline uses the `b` instruction to enter `HardFault`,
       so must be placed close to it. */
    *(.HardFaultTrampoline);
    *(.HardFault.*);
    . = ALIGN(4); /* Pad .text to the alignment to workaround overlapping load section bug in old lld */
    __etext = .;
  } > REGION_TEXT AT> REGION_LOAD_TEXT
  __sitext = LOADADDR(.text);

  .rodata : ALIGN(4)
  {
    FILL(0xff);
    . = ALIGN(4);
    __srodata = .;
    *(.rodata .rodata.*);

    /* 4-byte align the end (VMA) of this section.
       This is required by LLD to ensure the LMA of the following .data
       section will have the correct alignment. */
    . = ALIGN(4);
    __erodata = .;
  } > REGION_RODATA AT> REGION_LOAD_RODATA
  __sirodata = LOADADDR(.rodata);

  .data : ALIGN(4)
  {
    FILL(0xff);
    . = ALIGN(4);
    __sdata = .;
    *(.data .data.*);
    . = ALIGN(4); /* 4-byte align the end (VMA) of this section */
    __edata = .;
  } > REGION_DATA AT> REGION_LOAD_DATA
  __sidata = LOADADDR(.data);

  .bss (NOLOAD) : ALIGN(4)
  {
    . = ALIGN(4);
    __sbss = .;
    *(.bss .bss.*);
    *(COMMON); /* Uninitialized C statics */
    . = ALIGN(4); /* 4-byte align the end (VMA) of this section */
      __ebss = .;
  } > REGION_BSS

  .uninit (NOLOAD) : ALIGN(4)
  {
    . = ALIGN(4);
    __suninit = .;
    *(.uninit .uninit.*);
    . = ALIGN(4);
    __euninit = .;
  } > REGION_UNINIT

  .heap (NOLOAD) : ALIGN(4)
  {
    __sheap = .;
    . += ALIGN(__heap_size, 4);
    __eheap = .;
  } > REGION_HEAP

  /* Dynamic relocations are unsupported. This section is only used to detect relocatable code in
     the input files and raise an error if relocatable code is found */
  .got (NOLOAD) :
  {
    KEEP(*(.got .got.*));
  }

  /DISCARD/ :
  {
    /* Unused exception related info that only wastes space */
    *(.ARM.exidx);
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }
}

/* Do not exceed this mark in the error messages below                                    | */
/* # Alignment checks */

ASSERT(__sstack % 8 == 0 && __estack % 8 == 0, "
BUG(imxrt-rt): .stack is not 8-byte aligned");

ASSERT(__sdata % 4 == 0 && __edata % 4 == 0, "
BUG(imxrt-rt): .data is not 4-byte aligned");

ASSERT(__sidata % 4 == 0, "
BUG(imxrt-rt): the LMA of .data is not 4-byte aligned");

ASSERT(__sbss % 4 == 0 && __ebss % 4 == 0, "
BUG(imxrt-rt): .bss is not 4-byte aligned");

ASSERT(__sheap % 4 == 0, "
BUG(imxrt-rt): start of .heap is not 4-byte aligned");

/* # Position checks */

/* ## .vector_table */
ASSERT(__reset_vector == ADDR(.vector_table) + 0x8, "
BUG(imxrt-rt): the reset vector is missing");

ASSERT(__eexceptions == ADDR(.vector_table) + 0x40, "
BUG(imxrt-rt): the exception vectors are missing");

ASSERT(SIZEOF(.vector_table) > 0x40, "
ERROR(imxrt-rt): The interrupt vectors are missing.
Possible solutions, from most likely to less likely:
- Link to imxrt-ral, or another compatible device crate
- Check that you actually use the device/hal/bsp crate in your code
- Disable the 'device' feature of cortex-m-rt to build a generic application (a dependency
may be enabling it)
- Supply the interrupt handlers yourself. Check the documentation for details.");

/* # Other checks */
ASSERT(SIZEOF(.got) == 0, "
ERROR(imxrt-rt): .got section detected in the input object files
Dynamic relocations are not supported. If you are linking to C code compiled using
the 'cc' crate then modify your build script to compile the C code _without_
the -fPIC flag. See the documentation of the `cc::Build.pic` method for details.");

ASSERT((__dcd_end - __dcd_start) % 4 == 0, "
ERROR(imxrt-rt): .dcd (Device Configuration Data) size must be a multiple of 4 bytes.");
/* Do not exceed this mark in the error messages above                                    | */

/* ===--- End imxrt-link.x ---=== */
