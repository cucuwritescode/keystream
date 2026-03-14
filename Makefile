BINARY_NAME = keystream
INSTALL_DIR = $(HOME)/.local/bin

.PHONY: install uninstall clean

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

uninstall:
	rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "  removed $(BINARY_NAME)"

clean:
	cargo clean
