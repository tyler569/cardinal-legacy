
target remote localhost:1234

symbol-file ./cardinal.elf
set architecture i386:x86-64

break start_higher_half
break break_point

continue

