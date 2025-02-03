/* ===--- Begin imxrt-boot-header-1180.x ---===
 * This extra content is injected into the linker script depending on the
 * runtime configuration.
 */

__image_size = SIZEOF(.vector_table) + SIZEOF(.text) + SIZEOF(.xip) + SIZEOF(.rodata) + SIZEOF(.data);

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
   * Specifically, point the 'Image offset' to somewhere other
   * than the start of flash, and specify how many bytes to copy.
   * The boot ROM copies the image, then jumps to the entry point.
   * It is currently not tested or used, mainly  for consistency
   * with the other iMXRT families.
   */
  .boot1 ORIGIN(FLASH):
  {
    FILL(0x00);
    /* ------------------
     * Memory configuration block
     * ------------------
     *
     * The size and layout is different for different boot devices. Currently,
     * only NOR flash is supported.
     */
    . += __fcb_offset;          /* Can change based on boot source */
    KEEP(*(.fcb));
    . = ORIGIN(FLASH) + 0x1000;

    /* ------------------
     * Container 1
     * ------------------
     */
    __container1_start = .;
    LONG(0x87000000 | (__container1_len << 8)); /* Tag, length, version */
    LONG(0); /* Flags */
    LONG(0x01000000); /* 1 image, fuse version 0, SW version 0 */
    LONG(__signature_block_start - __container1_start); /* Signature block offset */

    /* Image array, image 0 */
    LONG(0xa000); /* Image offset */
    LONG(__image_size); /* Image size */
    QUAD(LOADADDR(.vector_table)); /* Load address (execute in place) */
    QUAD(Reset); /* Entry point */
    LONG(0x213); /* Flags: Unencrypted, SHA512 hashed, executable image for Cortex-M33 */
    LONG(0); /* Reserved (image meta data) */
  } > FLASH

  /* Put the hash in a separate section for easier replacement in a post-build step */
  .image_hash :
  {
    QUAD(0); /* Hash 512 bytes */
    QUAD(0);
    QUAD(0);
    QUAD(0);
    QUAD(0);
    QUAD(0);
    QUAD(0);
    QUAD(0);
  } > FLASH

  .boot2 :
  {
    FILL(0x00);
    QUAD(0); /* IV 256 bytes, zero for unencrypted image */
    QUAD(0);
    QUAD(0);
    QUAD(0);

    /* ------------------
     * Signature block
     * ------------------
     */
    __signature_block_start = .;
    LONG(0x90000000 | (__signature_block_len << 8)); /* Tag, length, version */
    LONG(0); /* SRK Table offset, Certificate Offset */
    LONG(0); /* Signature offset, Blob offset */
    LONG(0); /* Reserved */
    __signature_block_end = .;
    __signature_block_len = __signature_block_end - __signature_block_start;
    __container1_end = .;
    __container1_len = __container1_end - __container1_start;

    /* DCD is replaced by XMCD. Satisfy assertions intended for other families. */
    __dcd_start = .;
    __dcd_end = .;

    . = ORIGIN(FLASH) + 0x2000;   /* Pad to 8k alignment for the container/images boundary */
  } > FLASH
}

ASSERT((__dcd_end - __dcd_start) % 4 == 0, "
ERROR(imxrt-rt): .dcd (Device Configuration Data) size must be a multiple of 4 bytes.");

/* ===--- End imxrt-boot-header-1180.x ---=== */
