
.PHONY: all
all: queens.js
	@echo > /dev/null

queens.js: queens.rs
	rustc --target=asmjs-unknown-emscripten $< -o $@

.PHONY: test
test: queens
	./queens

queens: queens.rs
	rustc --test $< -o $@


.PHONY: setup
setup:
	curl -O https://s3.amazonaws.com/mozilla-games/emscripten/releases/emsdk-portable.tar.gz
	tar -xzf emsdk-portable.tar.gz
	./emsdk_portable/emsdk install sdk-incoming-64bit
	./emsdk_portable/emsdk activate sdk-incoming-64bit
