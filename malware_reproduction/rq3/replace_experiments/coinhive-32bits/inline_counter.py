import os
import re
import sys
from subprocess import call

inlineAdd = " (set_global 5 (i32.add (get_global 5) (i32.const 1)))"
inlineAnd = " (set_global 6 (i32.add (get_global 6) (i32.const 1)))"
inlineShl = " (set_global 7 (i32.add (get_global 7) (i32.const 1)))"
inlineShr = " (set_global 8 (i32.add (get_global 8) (i32.const 1)))"
inlineXor = " (set_global 9 (i32.add (get_global 9) (i32.const 1)))"

rAdd = re.compile("\s*\(i32.add\s*$")
rAnd = re.compile("\s*\(i32.and\s*$")
rShl = re.compile("\s*\(i32.shl\s*$")
rShr = re.compile("\s*\(i32.shr_u\s*$")
rXor = re.compile("\s*\(i32.xor\s*$")

wat2wasm = "../wabt/wat2wasm.exe"
inFileName = str(sys.argv[1])

baseName = os.path.splitext(inFileName)[0]
outFileName = baseName + "_profiled.wat"
outWasm = baseName + "_profiled.wasm"

with open(inFileName) as inFile:
	lines = inFile.read().splitlines()

outLines = []

for line in lines:
	if rAdd.match(line):
		line += inlineAdd
	elif rAnd.match(line):
		line += inlineAnd
	elif rShl.match(line):
		line += inlineShl
	elif rShr.match(line):
		line += inlineShr
	elif rXor.match(line):
		line += inlineXor
	outLines.append(line)

with open(outFileName, 'w+') as outFile:
	for line in outLines:
		outFile.write("%s\n" % line)

# wat --> wasm
call([wat2wasm, outFileName, "-o", outWasm])