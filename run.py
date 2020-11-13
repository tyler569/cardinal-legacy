#!/usr/bin/env python3

import argparse
import os
import subprocess

parser = argparse.ArgumentParser(description='Convenient qemu wrapper')
parser.add_argument('-f', '--file', help='ISO to run (default cardinal.iso)', default='cardinal.iso')
parser.add_argument('-r', '--ram', help="Set the VM's RAM size", default='32M')
parser.add_argument('-d', '--debug', action='store_true', help='Wait for GDB debug connection')
parser.add_argument('-v', '--video', action='store_true', help='Show video')
parser.add_argument('-i', '--interrupts', action='store_true', help='Show interrupt debug information')
parser.add_argument('-m', '--monitor', action='store_true', help='Show the QEMU monitor on stdio (implies -t)')
parser.add_argument('-x', '--net', action='store_true', help='Attach a network interface')
parser.add_argument('-n', '--no-serial', action='store_false', help='Do not use serial stdio', dest='serial', default=True)
parser.add_argument('-t', '--no-tee', action='store_false', help='Do not tee output', dest='tee', default=True)
parser.add_argument('--debugcon', action='store_true', help='Enable QEMU debug console (port E9)')
parser.add_argument('--test-mode', action='store_true', help='Run in test mode (attach isa-debug-exit device)')
parser.add_argument('--dry-run', action='store_true', help="Just print the QEMU command, don't run it")
options = parser.parse_args()

qemu_command = ['qemu-system-x86_64']
qemu_command.append('-s')
qemu_command.append('-vga std')
qemu_command.append('-no-reboot')
qemu_command.append(f'-m {options.ram}')
qemu_command.append(f'-cdrom {options.file}')

if options.debug:
    qemu_command.append('-S')
if options.monitor:
    qemu_command.append('-monitor stdio')
if options.serial and not options.monitor:
    qemu_command.append('-serial stdio')
if options.debugcon:
    qemu_command.append('-debugcon stdio')
if options.interrupts:
    qemu_command.append('-d int')
if not options.video:
    qemu_command.append('-display none')
if options.test_mode:
    qemu_command.append('--device isa-debug-exit')

qemu_command.append('-serial unix:./serial2,nowait,server')

if options.net:
    qemu_command.append('-device rtl8139,netdev=net0')
    qemu_command.append('-netdev tap,id=net0,script=no,downscript=no,ifname=tap0')
    qemu_command.append('-object filter-dump,id=dump0,netdev=net0,file=tap0.pcap')

if options.tee and not options.monitor:
    qemu_command.append('| tee last_output')

qemu_command = ' '.join(qemu_command)
print(qemu_command)

if options.dry_run:
    exit()

try:
    subprocess.run(qemu_command, shell=True)
except KeyboardInterrupt:
    exit()
