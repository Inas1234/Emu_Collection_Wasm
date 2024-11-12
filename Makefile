EMULATORS := chip8 
WEB_DIR := ../../web

all: $(EMULATORS)

$(EMULATORS):
	@echo "Building $@ with wasm-pack..."
	wasm-pack build emulators/$@ --release --target web --out-dir $(WEB_DIR)/$@

clean:
	@echo "Cleaning up..."
	cargo clean
	rm -rf $(WEB_DIR)/*

serve:
	@echo "Serving on http://localhost:8080"
	python3 -m http.server 8080 --directory $(WEB_DIR)

.PHONY: all clean serve $(EMULATORS)
