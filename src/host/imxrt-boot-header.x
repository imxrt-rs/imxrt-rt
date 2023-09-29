/* ===--- Begin imxrt-boot-header.x ---===
 * This extra content is injected into the linker script depending on the
 * runtime configuration.
 */

/* If you're ever playing with the boot ROM copy, this is your image size.
 *
 * Note that it depends on the section layout! Need to represent contiguous
 * sections starting from the boot header.
 */
__image_size = SIZEOF(.boot) + SIZEOF(.vector_table) + SIZEOF(.text) + SIZEOF(.rodata);

/* END TODO */
EXTERN(FLEXSPI_CONFIGURATION_BLOCK);

/* # Sections */
SECTIONS
{
  /* Boot header for serial NOR FlexSPI XIP.
   *
   * It's 'XIP' in that it starts executing instructions
   * from flash immediately out of reset. The runtime then
   * manually copies instructions (data, etc.), and we jump
   * to that. After that jump, we're no longer XIP.
   *
   * The i.MX RT boot ROM also supports a way to copy the
   * application image by changing the boot data configuration.
   * Specifically, point the 'start of image' to somewhere other
   * than the start of flash, and specify how many bytes to copy.
   * The boot ROM copies the image, then jumps to the vector table.
   * There's a catch: the boot ROM copies the first 8K from the
   * start of flash too. This represents the entire boot header,
   * including the FCB, IVT, and boot data. (NXP docs say that the
   * initial load region is 4K; my testing shows that it's 8K, and
   * this aligns with observations of others.) If you ever want to
   * try this, make sure you're specifing the VMA and LMA of the
   * boot head section to represent this 8K relocation.
   */
  .boot ORIGIN(FLASH):
  {
    . += __fcb_offset;          /* Changes based on the chip */
    KEEP(*(.fcb));
    . = ORIGIN(FLASH) + 0x1000;
    /* ------------------
     * Image vector table
     * ------------------
     *
     * Not to be confused with the ARM vector table. This tells the boot ROM
     * where to find the boot data and (eventual) first vector table.
     * The IVT needs to reside right here.
     */
    __ivt = .;
    LONG(0x402000D1);           /* Header, magic number */
    LONG(__sivector_table);     /* Address of the vectors table */
    LONG(0x00000000);           /* RESERVED */
    LONG(__dcd);                /* Device Configuration Data */
    LONG(__boot_data);          /* Address to boot data */
    LONG(__ivt);                /* Self reference */
    LONG(0x00000000);           /* Command Sequence File (unused) */
    LONG(0x00000000);           /* RESERVED */
    /* ---------
      * Boot data
      * ---------
      */
    __boot_data = .;
    LONG(ORIGIN(FLASH));        /* Start of image */
    LONG(__image_size);         /* Length of image */
    LONG(0x00000000);           /* Plugin flag (unused) */
    LONG(0xDEADBEEF);           /* Dummy to align boot data to 16 bytes */
    . = ALIGN(4);
    __dcd_start = .;
    KEEP(*(.dcd));              /* Device Configuration Data */
    __dcd_end = .;
    __dcd = ((__dcd_end - __dcd_start) > 0) ? __dcd_start : ABSOLUTE(0);
    *(.Reset);                  /* Jam the imxrt-rt reset handler into flash. */
    *(.__pre_init);             /* Also jam the pre-init function, since we need it to run before instructions are placed. */
    . = ORIGIN(FLASH) + 0x2000;   /* Reserve the remaining 8K as a convenience for a non-XIP boot. */
  } > FLASH
}

/* ===--- End imxrt-boot-header.x ---=== */
