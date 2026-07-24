APP := tuipe
BIN := target/release/tuipe
VERSION := $(shell printf "r%s.%s" "$$(git rev-list --count HEAD)" "$$(git rev-parse --short HEAD)")

RS_FILES := $(shell find src/ -name '*.rs')

PREFIX ?= /usr
DESTDIR ?=
BINDIR := $(DESTDIR)$(PREFIX)/bin
DATADIR := $(DESTDIR)$(PREFIX)/share/$(APP)

.PHONY: build install uninstall clean run

build: $(BIN)

$(BIN): $(RS_FILES) Cargo.lock Cargo.toml
	@printf "\e[36m==> \e[0mCompiling...\n"
	@DATADIR=$(DATADIR) VERSION=$(VERSION) cargo build --release
	@printf "[\033[32m OK \033[0m] Build complete\n"

run:
	@mkdir -p $$HOME/.local/share/tuipe
	@cargo run

install: $(BIN)
	@printf "\033[36m==> \033[0mInstalling files...\n"
	@mkdir -p $(BINDIR)
	install -m755 $(BIN) $(BINDIR)/$(APP)

	@mkdir -p $(DATADIR)/languages
	@files=(languages/*); \
	for file in "$${files[@]}"; do \
		install -m644 "$$file" $(DATADIR)/"$$file"; \
		echo "install -m644 $$file $(DATADIR)/$$file"; \
	done
	@mkdir -p $$HOME/.local/share/tuipe
	@printf "[\033[32m OK \033[0m] Installation complete\n"

uninstall:
	@printf "\033[36m==> \033[0mRemoving installed files...\n"
	rm -f $(BINDIR)/$(APP)
	rm -rf $(DATADIR)
	@printf "[\033[32m OK \033[0m] Uninstall complete\n"

clean:
	@printf "\033[36m==> \033[0mRemoving build artifacts...\n"
	@cargo clean
	@printf "[\033[32m OK \033[0m] Clean complete\n"

