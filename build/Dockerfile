FROM debian:latest

# Update package index and install necessary packages
RUN apt-get update && \
    apt-get install -y \
    nasm \
    rustc \
    build-essential \
    grub-common \
    xorriso \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install grub-pc-bin
RUN apt-get update && \
    apt-get install -y grub-pc-bin \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Set environment variables
ENV PATH="/root/.cargo/bin:${PATH}"

# Create a working directory
RUN mkdir /kfs
WORKDIR /kfs

# Copy the necessary files into the container
COPY . .

# Ensure the script is executable
RUN chmod +x build_kernel.sh

# Specify the default command to run
CMD ["/bin/bash", "./build/build_kernel.sh"]