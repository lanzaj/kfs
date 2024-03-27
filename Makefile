DOCKER_IMAGE := kernel_builder
ISO_FILE := kfs.iso

all: $(ISO_FILE)

$(ISO_FILE): build_image
	@echo "Creating kernel iso image..."
	docker run -v $(CURDIR):/kfs $(DOCKER_IMAGE)
	rm -f src/boot.o src/multiboot_header.o isofiles/boot/kernel.bin

build_image:
	@if [ -z $$(docker images -q $(DOCKER_IMAGE)) ]; then \
		echo "Building docker image for kernel building..."; \
		docker build -t $(DOCKER_IMAGE) ./build ; \
	else \
		echo "Docker image $(DOCKER_IMAGE) already exists. Skipping Docker build..."; \
	fi

clean:
	@echo "Cleaning up..."
	rm -f src/boot.o src/multiboot_header.o $(ISO_FILE)

fclean: clean
	docker system prune -af

run: $(ISO_FILE)
	@echo "Launching KFS..."
	kvm -cpu host -cdrom $(ISO_FILE)

.PHONY: all clean run fclean build_image
