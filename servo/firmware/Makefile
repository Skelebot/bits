TARGET_DIR=target/thumbv6m-none-eabi/release
BIN_NAME=firmware

all: build objcopy flash

build:
	cargo build --release

objcopy:
	cargo objcopy --release --bin firmware -- -O binary $(TARGET_DIR)/$(BIN_NAME).bin

size:
	#cargo objcopy -O binary $(TARGET_DIR)/$(BIN_NAME) $(TARGET_DIR)/$(BIN_NAME).bin
	cargo size --release --bin firmware -- -O binary $(TARGET_DIR)/$(BIN_NAME).bin
	
flash:
	sudo dfu-util -a 0 -s 0x08000000:leave -D $(TARGET_DIR)/$(BIN_NAME).bin
	
size:
	arm-none-eabi-size $(TARGET_DIR)/$(BIN_NAME)
