
CKB_DEBUGGER := ckb-debugger

simulator:
	cargo build --features=simulator --bin alt-bn128-example-simulator --release

rvv-asm:
	cargo build --features=use_rvv_vector --bin alt-bn128-example-rvv-asm --release

build: simulator rvv-asm

run-simulator:
	RUST_LOG=debug ${CKB_DEBUGGER} --max-cycles 2000000000 --bin ./target/riscv64imac-unknown-none-elf/release/alt-bn128-example-simulator

run-rvv-asm:
	RUST_LOG=debug ${CKB_DEBUGGER} --max-cycles 2000000000 --bin ./target/riscv64imac-unknown-none-elf/release/alt-bn128-example-rvv-asm

run: run-simulator run-rvv-asm


expand:
	cargo expand --lib
