.PHONY: build clean

build:
	tinygo build -o {{plugin_name}}.wasm -target wasm main.go

clean:
	rm -f {{plugin_name}}.wasm

install: build
	mkdir -p ~/.config/kargo/plugins
	cp {{plugin_name}}.wasm ~/.config/kargo/plugins/

test:
	go test ./...