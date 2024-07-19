FROM conchoid/docker-pyenv:v2.4.4-1-3.11-buster

WORKDIR /code

COPY requirements.txt /code/requirements.txt
RUN apt-get install -y libgmp3-dev
RUN yes | pyenv install 3.9.15 && pyenv global 3.9.15 && \
    pip install -r /code/requirements.txt

RUN mkdir /opt/app
RUN mkdir /opt/app/gen

RUN git clone https://github.com/starkware-libs/cairo-lang.git /opt/app/cairo-lang
RUN cd /opt/app/cairo-lang && git checkout efa9648f57568aad8f8a13fbf027d2de7c63c2c0
RUN cp -r /opt/app/cairo-lang/src/starkware /opt/app/starkware

COPY hidden/simple-bootloader-utils.py /opt/app/starkware/cairo/bootloaders/simple_bootloader/utils.py
COPY hidden/simple-bootloader-objects.py /opt/app/starkware/cairo/bootloaders/simple_bootloader/objects.py
COPY hidden/bootloader-utils.py /opt/app/starkware/cairo/bootloaders/bootloader/utils.py
COPY hidden/bootloader-objects.py /opt/app/starkware/cairo/bootloaders/bootloader/objects.py
COPY cpu_air_prover_config.json /opt/app/cpu_air_prover_config.json
COPY cpu_air_params.json /opt/app/cpu_air_params.json
COPY test_compiled_bootloader.json /opt/app/test_compiled_bootloader.json
COPY fact_topologies.json /opt/app/fact_topologies.json
COPY fibonacci_compiled.json /opt/app/fibonacci_compiled.json
COPY fibonacci_input.json /opt/app/fibonacci_input.json
COPY test_bootloader_fib.py /opt/app
RUN cd /opt/app && python3 test_bootloader_fib.py > output.log