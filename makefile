
BUILDMODE ?= debug

CC := x86_64-elf-gcc

RUSTSRC := $(shell find src -name '*.rs')
ASMSRC := $(shell find asm -name '*.asm')

ASMOBJ := $(patsubst %.asm,%.o,$(ASMSRC))

OBJECTS := $(ASMOBJ)
RUSTLIB := target/x86_64-cardinal/$(BUILDMODE)/libcardinal.a

.PHONY: all clean

all: cardinal.iso

%.o: %.asm
	nasm -felf64 -o $@ $<

$(RUSTLIB): $(RUSTSRC) Cargo.toml
	cargo xbuild

cardinal.elf: $(ASMOBJ) $(RUSTLIB)
	ld -g -nostdlib -o $@ -T link.ld $(ASMOBJ) $(RUSTLIB)

cardinal.iso: cardinal.elf grub.cfg
	mkdir -p isodir/boot/grub
	cp grub.cfg isodir/boot/grub
	cp cardinal.elf isodir/boot/
	grub-mkrescue -o $@ isodir/
	rm -rf isodir

clean:
	rm -f asm/*.o
	rm -f cardinal.elf
	rm -f nimos.iso

