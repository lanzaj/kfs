FROM debian:latest

# Update package index and install necessary packages
RUN apt-get update && \
    apt-get install -y \
    nasm \
    curl \
    build-essential \
    grub-common \
    xorriso \
    grub-pc-bin \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain nightly \
    --profile minimal \
    --no-modify-path

# Set environment variables
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install xargo

RUN rustup default nightly
RUN rustup component add rust-src

# Create a working directory
RUN mkdir /kfs
WORKDIR /kfs

# Copy the necessary files into the container
COPY . .

# Ensure the script is executable
RUN chmod +x build_kernel.sh

# Specify the default command to run
CMD ["/bin/bash", "./build/build_kernel.sh"]