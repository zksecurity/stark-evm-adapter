FROM ciimage/python:3.9-ci

# install pyenv
RUN curl https://pyenv.run | bash
ENV HOME="/root"
WORKDIR ${HOME}
ENV PYENV_ROOT="${HOME}/.pyenv"
ENV PATH="${PYENV_ROOT}/shims:${PYENV_ROOT}/bin:${PATH}"

# install python 3.9.15
RUN yes | pyenv install 3.9.15 && pyenv global 3.9.15

# install Rust 1.74.1
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup install 1.74.1
RUN rustup default 1.74.1-x86_64-unknown-linux-gnu

# install foundry anvil
RUN curl -L https://foundry.paradigm.xyz | bash
RUN /root/.foundry/bin/foundryup
ENV PATH="/root/.foundry/bin:${PATH}"

# copy the project
RUN mkdir /opt/app
WORKDIR /opt/app
COPY examples ./examples/
COPY Cargo.toml Cargo.lock README.md ./ 
COPY src ./src/
COPY tests ./tests/

# install required python dependencies
RUN pip install -r examples/bootloader/requirements.txt

# install cairo-lang
WORKDIR /opt/app/examples/bootloader
RUN git clone https://github.com/starkware-libs/cairo-lang.git cairo-lang
RUN cd cairo-lang && git checkout efa9648f57568aad8f8a13fbf027d2de7c63c2c0 && cd ..
RUN cp -r cairo-lang/src/starkware starkware/
RUN rm -rf cairo-lang/
RUN cp hidden/simple-bootloader-utils.py starkware/cairo/bootloaders/simple_bootloader/utils.py
RUN cp hidden/simple-bootloader-objects.py starkware/cairo/bootloaders/simple_bootloader/objects.py
RUN cp hidden/bootloader-utils.py starkware/cairo/bootloaders/bootloader/utils.py
RUN cp hidden/bootloader-objects.py starkware/cairo/bootloaders/bootloader/objects.py

# build the Rust project
WORKDIR /opt/app
RUN cargo install --path .

# generate the annotated proof
WORKDIR /opt/app/examples/bootloader
RUN python3 test_bootloader_fib.py > output.log

# build the verify_stone_proof example
WORKDIR /opt/app
RUN cargo build --example verify_stone_proof

CMD ["sh", "-c", "cargo run --example verify_stone_proof"]