EMULATORS := chip8 gameboy 
WEB_DIR := ../../web
DOCS_DIR := ./docs

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
	python3 -m http.server 8080 --directory ./web

copy:
	@echo "Copying files to docs folder..."
	cp -r ./web/* $(DOCS_DIR)/ 2>/dev/null || true
	@echo "Done"


.PHONY: all clean serve $(EMULATORS)
