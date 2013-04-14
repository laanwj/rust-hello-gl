RUSTC ?= rustc
RUSTFLAGS ?= -L ../rust-sdl -L ../rust-opengles

.PHONY: all
all: hello-gl

hello-gl: hello-gl.rs
	$(RUSTC) $(RUSTFLAGS) $< -o $@

.PHONY: clean
clean:
	rm -f hello-gl
