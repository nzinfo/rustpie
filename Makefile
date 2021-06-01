
ARCH ?= aarch64
PROFILE ?= release
USER_PROFILE ?= release

# NOTE: this is to deal with `(signal: 11, SIGSEGV: invalid memory reference)`
# https://github.com/rust-lang/rust/issues/73677
RUSTFLAGS := -C llvm-args=-global-isel=false

# NOTE: generate frame pointer for every function
export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

ifeq (${PROFILE}, release)
CARGO_FLAGS := ${CARGO_FLAGS} --release
endif

ifeq (${USER_PROFILE}, release)
CARGO_FLAGS := ${CARGO_FLAGS} --features user_release
endif

USER_IMAGE := user/target/${ARCH}/${USER_PROFILE}/rustpi-user

KERNEL := target/${ARCH}/${PROFILE}/rustpi

.PHONY: all emu debug dependencies ${USER_IMAGE} clean

all: ${KERNEL} ${KERNEL}.bin ${KERNEL}.asm

${KERNEL}: ${USER_IMAGE}
	cargo build --target src/target/${ARCH}.json -Z build-std=core,alloc ${CARGO_FLAGS}

${USER_IMAGE}:
	make ARCH=${ARCH} USER_PROFILE=${USER_PROFILE} -C user

${KERNEL}.bin: ${KERNEL}
	llvm-objcopy $< -O binary $@

${KERNEL}.asm: ${KERNEL}
	llvm-objdump -d $< > $@

ifeq (${ARCH}, aarch64)
QEMU_CMD := qemu-system-aarch64 -M virt -cpu cortex-a53
endif
ifeq (${ARCH}, riscv64)
QEMU_CMD := qemu-system-riscv64 -M virt -bios default
endif

QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
					 -global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

emu: ${KERNEL}.bin
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} ${QEMU_DISK_OPTIONS} -kernel $<

debug: ${KERNEL}.bin
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} ${QEMU_DISK_OPTIONS} -kernel $< -s -S

clean:
	-cargo clean
	make -C user clean

dependencies:
	rustup component add rust-src