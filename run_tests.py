#!/usr/bin/env python3

import os
import subprocess as sp


def compile(path, show_diagnostics):
	try:
		output = None if show_diagnostics else sp.DEVNULL
		sp.check_call('./latc_llvm {}'.format(path), shell=True, stderr=output)
		return True
	except sp.CalledProcessError:
		return False


def run(path):
	inp = path.replace('.lat', '.input')
	out = path.replace('.lat', '.output')
	bc = path.replace('.lat', '.bc')
	stdin = open(inp) if os.path.exists(inp) else None
	actual_out = sp.check_output('lli {}'.format(bc), shell=True, stdin=stdin).decode('ascii')

	# No output
	if not os.path.isfile(out):
		return True

	wanted_out = open(out).read()
	if actual_out == wanted_out:
		return True

	return False


def list_latte_files(base):
	return sorted(map(lambda x: os.path.join(base, x), filter(lambda x: x.endswith('.lat'), os.listdir(base))))


def test_positive(base):
	for path in list_latte_files(base):
		if not compile(path, True):
			print('{}'.format(open(path).read()))
			print('{} ERROR <=========================================='.format(path))
			continue

		if run(path):
			print('{} OK'.format(path))
		else:
			print('{}'.format(open(path).read()))
			print('{} ERROR <=========================================='.format(path))


def test_negative(base):
	compiled = {}
	for path in list_latte_files(base):
		compiled[path] = compile(path, False)
		if not compiled[path]:
			print('{} => OK'.format(path))
		else:
			print('{}'.format(open(path).read()))
			print('{} ERROR <=========================================='.format(path))


if __name__ == '__main__':
	test_positive('./lattests/good')
	test_negative('./lattests/bad')
	test_negative('./mrjp-tests/bad/semantic/')
	test_negative('./mrjp-tests/bad/runtime/')
	test_negative('./mrjp-tests/bad/infinite_loop/')
	test_positive('./mrjp-tests/good/basic/')


