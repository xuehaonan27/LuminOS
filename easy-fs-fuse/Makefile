TARGET := riscv64gc-unknown-none-elf
MODE := release
USER_DIR := ../user
SRC_DIR := $(USER_DIR)/src/bin/
TARGET_DIR := $(USER_DIR)/target/$(TARGET)/$(MODE)/
VOLUME_MIB := 16
FS_IMG := fs.img

build:
	@cd ../user && make clean && make build
	@cargo clean
	@cargo build --release
	@cargo run -- -s $(SRC_DIR) -t $(TARGET_DIR) -v $(VOLUME_MIB)
	@echo "made fs.img"

clean:
	@cargo clean