# Nome do binário gerado
BIN=target/release/ts
BIN_DEBUG=target/debug/ts

# Build de produção (estático, sem dynamic_linking)
release:
	cargo build --release

# Build de produção (estático, sem dynamic_linking) (Android)
android:
	cargo ndk -t arm64-v8a -o ./target build --release
	
# Build de desenvolvimento (com dynamic_linking)
debug:
	cargo run --features with_dynamic

# Executar release (sem dynamic_linking)
run-release: release
	./$(BIN)

# Limpar o projeto
clean:
	cargo clean
