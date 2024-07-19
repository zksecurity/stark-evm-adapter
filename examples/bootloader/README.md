# Test bootloader
Test bootloader with `cairo-run` using fibonacci program as example

## Prerequisites
- Copy `objects.py` and `utils.py` for bootloader as `hidden/bootloader-objects.py` and `hidden/bootloader-utils.py`
- Copy `objects.py` and `utils.py` for simple bootloader as `hidden/simple-bootloader-objects.py` and `hidden/simple-bootloader-utils.py`

## Docker
### Run
- Run `docker build --tag test .`
### Check output
- `container_id=$(docker create test)`
- `docker cp -L ${container_id}:/opt/app/output.log .`
- Check `output.log`

## Makefile
Note: requires python 3.9.15 and installing [required packages](https://github.com/starkware-libs/cairo-lang/blob/master/scripts/requirements.txt), also may need to follow [installation instructions](https://docs.cairo-lang.org/quickstart.html)
- Run `make deps` to install dependencies
- Run `make test` to build and run test
