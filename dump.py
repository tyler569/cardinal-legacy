#!/usr/bin/env python3

import optparse
import os
import re
import subprocess

parser = optparse.OptionParser()
parser.add_option('-a', '--addr2line', action='store_true', help='Run addr2line on last_output')
parser.add_option('-s', '--source', action='store_true', help="Intersperce source (default)", default=True, dest='source')
parser.add_option('-S', '--no-source', action='store_false', help="Don't intersperce source", dest='source')
parser.add_option('-f', '--file', help='Program to dump', default='NGK')
parser.add_option('-i', '--intel', action='store_true', help='Dump in intel-format asm')
parser.add_option('-t', '--att', action='store_false', help='Dump in att-format asm', dest='intel')
parser.add_option('-x', '--rustfilt', action='store_true', help='Use rustfilt (default)', default=True)
parser.add_option('-X', '--no-rustfilt', action='store_false', help='Disable rustfilt', dest='rustfilt')

(options, args) = parser.parse_args()

bits = 64

file = options.file
if file == 'NGK':
    file = 'cardinal.elf' ### defaults

objdump = 'objdump'

if options.addr2line:
    output = subprocess.check_output('tail -n50 last_output', shell=True)
    output = output.decode("UTF-8")
    if "backtrace" not in output:
        print("No backtrace found")
        exit(0)
    addresses = []
    for line in output.split("\n"):
        m = re.search("\((.*)\) <.*>", line)
        if m:
            addresses.append(m.group(1))
        m = re.search(".+bp:.+ ip: (\W+)", line)
        if m:
            addresses.append(m.group(1))
    command = f'addr2line -fips -e {file} {" ".join(addresses)}'
    subprocess.run(command, shell=True)
    exit()

command = objdump
if options.source:
    command += ' -dS'
else:
    command += ' -d'
if options.intel:
    command += ' -Mintel'
command += ' -j.text -j.low.text'
command += f' {file}'
if options.rustfilt:
    command += ' | rustfilt'
command += ' | less'
subprocess.run(command, shell=True)

