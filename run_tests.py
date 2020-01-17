#!/usr/bin/env python3

import os

base = './mrjp-tests/good/basic/'
files = filter(lambda x: x.endswith('.lat'), os.listdir(base))

import subprocess as sp
for lat in files:
	inp = lat.replace('.lat', '.input')
	out = lat.replace('.lat', '.output')
	cmd = ['./latc_llvm', base + lat]
	if os.path.isfile(inp):
		cmd.append('<')
		cmd.append(base + inp)
	
	bc = lat.replace('.lat', '.bc')
	try:
		print('{}'.format(cmd))
		sp.check_call(' '.join(cmd), shell=True)
	except sp.CalledProcessError:
		print('{} ERR'.format(lat))
		continue
	
	actual_out = sp.check_output('lli {}'.format(base + bc), shell=True).decode('ascii')
	if os.path.isfile(out):
		wanted_out = open(out).read()
		if actual_out != wanted_out:
			print('{} ERR'.format(lat))
		else:
			print('{} OK'.format(lat))
	

