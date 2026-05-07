.PHONY: build install uninstall clean test

PREFIX    ?= /usr/local
BIN_DIR   ?= $(PREFIX)/bin
SERVICE_DIR ?= /etc/systemd/system
LIB_DIR   ?= /var/lib/uswitch
SHARE_DIR ?= /srv/ai-projects /opt/ai-core

build:
	cargo build --release

install: build
	install -Dm755 target/release/usw $(BIN_DIR)/usw
	install -Dm644 templates/ai-runtime@.service $(SERVICE_DIR)/ai-runtime@.service
	install -dm700 $(LIB_DIR)
	install -dm755 $(SHARE_DIR)
	systemctl daemon-reload

uninstall:
	rm -f $(BIN_DIR)/usw
	rm -f $(SERVICE_DIR)/ai-runtime@.service
	systemctl daemon-reload

test:
	cargo test

clean:
	cargo clean
