<<<<<<< HEAD
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
=======
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
>>>>>>> 56d9cf80e25c3b66073c1e646f13ade10f2235eb
	cargo clean
