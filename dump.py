#!/usr/bin/env python3

import argparse
import os
import re
import subprocess

parser = argparse.ArgumentParser(description='Convenient objdump wrapper')
parser.add_argument('-f', '--file', help='Program to dump', default='cardinal.elf')
parser.add_argument('-a', '--all-sections', help='Dupm all sections, not just .text', action='store_true')
options = parser.parse_args()

command = ['objdump']

if options.all_sections:
    command.append('-D')
else:
    command.append('-d')
    command.append('-j.text')
    command.append('-j.low.text')

command.append('-S')
# command.append('-Mintel')
command.append(options.file)
command.append(' | rustfilt')
command.append(' | less')

command = " ".join(command)

subprocess.run(command, shell=True)

