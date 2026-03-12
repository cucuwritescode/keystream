BINARY_NAME = keystream
INSTALL_DIR = /usr/local/bin
PLIST_DIR = ~/Library/LaunchAgents
PLIST_NAME = com.gauchodsp.keystream.plist

.PHONY: install uninstall clean

install:
	cargo build --release
	sudo cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	sudo chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "installed to $(INSTALL_DIR)/$(BINARY_NAME)"
	mkdir -p $(PLIST_DIR)
	cp $(PLIST_NAME) $(PLIST_DIR)/$(PLIST_NAME)
	chmod 644 $(PLIST_DIR)/$(PLIST_NAME)
	launchctl load $(PLIST_DIR)/$(PLIST_NAME)

uninstall:
	launchctl unload $(PLIST_DIR)/$(PLIST_NAME)
	sudo rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	rm -f $(PLIST_DIR)/$(PLIST_NAME)

clean:
	cargo clean
