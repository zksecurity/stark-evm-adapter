.PHONY: all deps test clean

deps: clean
	git clone https://github.com/starkware-libs/cairo-lang.git && cd cairo-lang && git checkout efa9648f57568aad8f8a13fbf027d2de7c63c2c0 && cd ../
	cp -r cairo-lang/src/starkware .
	rm -rf cairo-lang/
	cp hidden/simple-bootloader-utils.py starkware/cairo/bootloaders/simple_bootloader/utils.py
	cp hidden/simple-bootloader-objects.py starkware/cairo/bootloaders/simple_bootloader/objects.py
	cp hidden/bootloader-utils.py starkware/cairo/bootloaders/bootloader/utils.py
	cp hidden/bootloader-objects.py starkware/cairo/bootloaders/bootloader/objects.py

test:
	@python3 -c "import sys; print('Current Python version:', sys.version); assert sys.version_info[:3] == (3, 9, 15), 'Python version is not 3.9.15'"
	python3 test_bootloader_fib.py

clean:
	rm -rf starkware/
	rm -rf cairo-lang/