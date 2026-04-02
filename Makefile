BINARY_NAME = keystream
INSTALL_DIR = $(HOME)/.local/bin
PLIST_NAME = com.gauchodsp.keystream.plist
PLIST_DIR = $(HOME)/Library/LaunchAgents

.PHONY: install uninstall clean install-daemon uninstall-daemon check

install:
	cargo build --release
	@mkdir -p $(INSTALL_DIR)
	cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo ""
	@echo "  installed to $(INSTALL_DIR)/$(BINARY_NAME)"
	@echo ""
	@if ! echo "$$PATH" | grep -q "$(INSTALL_DIR)"; then \
		echo "  add to PATH:"; \
		echo "    export PATH=\"$(INSTALL_DIR):\$$PATH\""; \
		echo ""; \
	fi

install-daemon: install
	@mkdir -p $(PLIST_DIR)
	sed 's|__INSTALL_DIR__|$(INSTALL_DIR)|g' $(PLIST_NAME) > $(PLIST_DIR)/$(PLIST_NAME)
	launchctl load $(PLIST_DIR)/$(PLIST_NAME) 2>/dev/null || true
	@echo "  daemon installed and loaded"
	@echo ""

uninstall-daemon:
	-launchctl unload $(PLIST_DIR)/$(PLIST_NAME) 2>/dev/null
	rm -f $(PLIST_DIR)/$(PLIST_NAME)
	@echo "  daemon unloaded and removed"

uninstall: uninstall-daemon
	rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "  removed $(BINARY_NAME)"

check:
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo test
	cargo build --release
	@if ls scripts/*.sh >/dev/null 2>&1; then \
		shellcheck scripts/*.sh 2>/dev/null || echo "  shellcheck not installed, skipping"; \
	fi
	@echo ""
	@echo "  all checks passed"
	@echo ""

clean:
	cargo clean
