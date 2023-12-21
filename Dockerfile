FROM rust:1.74.1

# install foundry anvil
RUN curl -L https://foundry.paradigm.xyz | bash
RUN /root/.foundry/bin/foundryup
ENV PATH="/root/.foundry/bin:${PATH}"

WORKDIR /usr/app

COPY . .

RUN cargo build --example verify_stone_proof

CMD ["sh", "-c", "cargo run --example verify_stone_proof"]