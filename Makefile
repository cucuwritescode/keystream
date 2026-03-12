USER := $(shell whoami)
UID := $(shell id -u $(USER))
BINARY_NAME = keyStream
INSTALL_DIR = /Users/$(USER)/Applications
PLIST_DIR = ~/Library/LaunchAgents

.PHONY: install uninstall clean


install:
	cargo build --release
	sudo cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "built to $(INSTALL_DIR)/$(BINARY_NAME)"
	sudo cp com.gauchodsp.keyStream.plist $(PLIST_DIR)/com.gauchodsp.keyStream.plist
	chmod 644 ~/Library/LaunchAgents/com.gauchodsp.keyStream.plist
	launchctl load $(PLIST_DIR)/com.gauchodsp.keyStream
uninstall: 
	sudo rm -rf $(INSTALL_DIR)/$(BINARY_NAME)
	launchctl unload $(PLIST_DIR)/com.gauchodsp.keyStream
	rm -f $(PLIST_DIR)/com.gauchodsp.keyStream.plist
	sudo rm -rf target

clean: 
	cargo clean
