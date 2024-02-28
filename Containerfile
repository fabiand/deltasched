FROM fedora
WORKDIR /app

RUN sudo dnf install -y cargo jq python3 rust-std-static-wasm32-unknown-unknown.noarch
RUN cargo install wasm-pack
ENV PATH="${PATH}:/root/.cargo/bin/"

ADD Cargo.* /app
ADD src/ /app/src/
ADD assets/ index.html favicon.ico /app/

#RUN cargo build
RUN wasm-pack build --target web

ENTRYPOINT python3 -m http.server
