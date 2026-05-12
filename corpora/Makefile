# corpora/Makefile — download and verify standard benchmark corpora
#
# Usage:
#   make -C corpora silesia    # fetch + verify Silesia corpus
#   make -C corpora enwik8     # fetch + verify enwik8 (100 MB Wikipedia text)
#   make -C corpora calgary    # fetch + verify Calgary corpus
#   make -C corpora all        # fetch all three (~330 MB total)
#   make -C corpora verify     # re-verify SHA-256 of downloaded files
#   make -C corpora list       # print paths + sizes of downloaded files
#   make -C corpora clean      # remove /tmp/7zippy-corpora/
#   make -C corpora help       # show this message
#
# Files are stored in /tmp/7zippy-corpora/ and are never committed to the repo.
# SHA-256 hashes are checked against corpora/SHA256SUMS in this directory.

CORPORA_DIR := /tmp/7zippy-corpora

SILESIA_URL  := http://mattmahoney.net/dc/silesia.zip
ENWIK8_URL   := http://mattmahoney.net/dc/enwik8.zip
CALGARY_URL  := http://mattmahoney.net/dc/calgary.tar

# Locate SHA-256 tool (shasum on macOS, sha256sum on Linux)
SHA_CMD := $(shell if command -v shasum >/dev/null 2>&1; then echo "shasum -a 256"; elif command -v sha256sum >/dev/null 2>&1; then echo "sha256sum"; else echo "shasum -a 256"; fi)

# Path to SHA256SUMS relative to the Makefile (same directory)
SHASUMS := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))SHA256SUMS

.PHONY: all silesia enwik8 calgary verify list clean help

## all: download all three corpora (silesia, enwik8, calgary)
all: silesia enwik8 calgary

## silesia: download Silesia corpus to /tmp/7zippy-corpora/silesia/
silesia: $(CORPORA_DIR)/silesia/.extracted

$(CORPORA_DIR)/silesia.zip:
	@echo "==> Fetching Silesia corpus (~76 MB) ..."
	@mkdir -p $(CORPORA_DIR) && \
	curl -L --fail --silent --show-error --output $(CORPORA_DIR)/silesia.zip $(SILESIA_URL) && \
	echo "    download complete: $(CORPORA_DIR)/silesia.zip"

$(CORPORA_DIR)/silesia/.extracted: $(CORPORA_DIR)/silesia.zip
	@echo "==> Verifying and extracting Silesia ..."
	@expected=$$(grep "silesia.zip" $(SHASUMS) | grep -v "^#" | grep -v "TODO" | awk '{print $$1}'); \
	if [ -n "$$expected" ]; then \
	    actual=$$($(SHA_CMD) $(CORPORA_DIR)/silesia.zip | awk '{print $$1}'); \
	    if [ "$$actual" != "$$expected" ]; then \
	        echo "ERROR: SHA-256 mismatch for silesia.zip" >&2; \
	        echo "  expected: $$expected" >&2; \
	        echo "  actual:   $$actual" >&2; \
	        exit 1; \
	    fi; \
	    echo "    SHA-256 OK"; \
	else \
	    echo "    WARNING: no verified SHA-256 for silesia.zip (see corpora/SHA256SUMS TODO)"; \
	fi && \
	mkdir -p $(CORPORA_DIR)/silesia && \
	unzip -q -o $(CORPORA_DIR)/silesia.zip -d $(CORPORA_DIR)/silesia && \
	touch $(CORPORA_DIR)/silesia/.extracted && \
	echo "    extracted to $(CORPORA_DIR)/silesia/"

## enwik8: download enwik8 (100 MB Wikipedia text) to /tmp/7zippy-corpora/enwik8
enwik8: $(CORPORA_DIR)/enwik8/.extracted

$(CORPORA_DIR)/enwik8.zip:
	@echo "==> Fetching enwik8 corpus (~36 MB compressed, 100 MB uncompressed) ..."
	@mkdir -p $(CORPORA_DIR) && \
	curl -L --fail --silent --show-error --output $(CORPORA_DIR)/enwik8.zip $(ENWIK8_URL) && \
	echo "    download complete: $(CORPORA_DIR)/enwik8.zip"

$(CORPORA_DIR)/enwik8/.extracted: $(CORPORA_DIR)/enwik8.zip
	@echo "==> Verifying and extracting enwik8 ..."
	@expected=$$(grep "enwik8.zip" $(SHASUMS) | grep -v "^#" | grep -v "TODO" | awk '{print $$1}'); \
	if [ -n "$$expected" ]; then \
	    actual=$$($(SHA_CMD) $(CORPORA_DIR)/enwik8.zip | awk '{print $$1}'); \
	    if [ "$$actual" != "$$expected" ]; then \
	        echo "ERROR: SHA-256 mismatch for enwik8.zip" >&2; \
	        echo "  expected: $$expected" >&2; \
	        echo "  actual:   $$actual" >&2; \
	        exit 1; \
	    fi; \
	    echo "    SHA-256 OK"; \
	else \
	    echo "    WARNING: no verified SHA-256 for enwik8.zip (see corpora/SHA256SUMS TODO)"; \
	fi && \
	mkdir -p $(CORPORA_DIR)/enwik8 && \
	unzip -q -o $(CORPORA_DIR)/enwik8.zip -d $(CORPORA_DIR)/enwik8 && \
	touch $(CORPORA_DIR)/enwik8/.extracted && \
	echo "    extracted to $(CORPORA_DIR)/enwik8/"

## calgary: download Calgary corpus (~3 MB) to /tmp/7zippy-corpora/calgary/
calgary: $(CORPORA_DIR)/calgary/.extracted

$(CORPORA_DIR)/calgary.tar:
	@echo "==> Fetching Calgary corpus (~3 MB) ..."
	@mkdir -p $(CORPORA_DIR) && \
	curl -L --fail --silent --show-error --output $(CORPORA_DIR)/calgary.tar $(CALGARY_URL) && \
	echo "    download complete: $(CORPORA_DIR)/calgary.tar"

$(CORPORA_DIR)/calgary/.extracted: $(CORPORA_DIR)/calgary.tar
	@echo "==> Verifying and extracting Calgary ..."
	@expected=$$(grep "calgary.tar" $(SHASUMS) | grep -v "^#" | grep -v "TODO" | awk '{print $$1}'); \
	if [ -n "$$expected" ]; then \
	    actual=$$($(SHA_CMD) $(CORPORA_DIR)/calgary.tar | awk '{print $$1}'); \
	    if [ "$$actual" != "$$expected" ]; then \
	        echo "ERROR: SHA-256 mismatch for calgary.tar" >&2; \
	        echo "  expected: $$expected" >&2; \
	        echo "  actual:   $$actual" >&2; \
	        exit 1; \
	    fi; \
	    echo "    SHA-256 OK"; \
	else \
	    echo "    WARNING: no verified SHA-256 for calgary.tar (see corpora/SHA256SUMS TODO)"; \
	fi && \
	mkdir -p $(CORPORA_DIR)/calgary && \
	tar -xf $(CORPORA_DIR)/calgary.tar -C $(CORPORA_DIR)/calgary && \
	touch $(CORPORA_DIR)/calgary/.extracted && \
	echo "    extracted to $(CORPORA_DIR)/calgary/"

## verify: re-check SHA-256 of all already-downloaded corpus archives
verify:
	@echo "==> Verifying SHA-256 checksums ..."
	@ok=0; fail=0; skip=0; \
	for archive in silesia.zip enwik8.zip calgary.tar; do \
	    file=$(CORPORA_DIR)/$$archive; \
	    if [ ! -f "$$file" ]; then \
	        echo "  SKIP: $$archive (not downloaded)"; \
	        skip=$$((skip+1)); \
	        continue; \
	    fi; \
	    expected=$$(grep "$$archive" $(SHASUMS) | grep -v "^#" | grep -v "TODO" | awk '{print $$1}'); \
	    if [ -z "$$expected" ]; then \
	        echo "  SKIP: $$archive (no hash in SHA256SUMS — see TODO)"; \
	        skip=$$((skip+1)); \
	        continue; \
	    fi; \
	    actual=$$($(SHA_CMD) "$$file" | awk '{print $$1}'); \
	    if [ "$$actual" = "$$expected" ]; then \
	        echo "  OK:   $$archive"; \
	        ok=$$((ok+1)); \
	    else \
	        echo "  FAIL: $$archive"; \
	        echo "        expected: $$expected"; \
	        echo "        actual:   $$actual"; \
	        fail=$$((fail+1)); \
	    fi; \
	done; \
	echo ""; \
	echo "Results: $$ok OK, $$fail FAIL, $$skip skipped"; \
	if [ $$fail -gt 0 ]; then exit 1; fi

## list: print paths and sizes of all downloaded corpus files
list:
	@echo "==> Downloaded corpora in $(CORPORA_DIR):"
	@for archive in silesia.zip enwik8.zip calgary.tar; do \
	    file=$(CORPORA_DIR)/$$archive; \
	    if [ -f "$$file" ]; then \
	        size=$$(du -sh "$$file" | awk '{print $$1}'); \
	        echo "  $$file  ($$size)"; \
	    else \
	        echo "  $$file  (not downloaded)"; \
	    fi; \
	done
	@echo ""
	@echo "==> Extracted directories:"
	@for dir in silesia enwik8 calgary; do \
	    path=$(CORPORA_DIR)/$$dir; \
	    if [ -d "$$path" ]; then \
	        size=$$(du -sh "$$path" | awk '{print $$1}'); \
	        echo "  $$path/  ($$size)"; \
	    else \
	        echo "  $$path/  (not extracted)"; \
	    fi; \
	done

## clean: remove all downloaded corpora from /tmp/7zippy-corpora/
clean:
	@echo "==> Removing $(CORPORA_DIR) ..."
	@rm -rf $(CORPORA_DIR) && echo "    done."

## help: show available targets
help:
	@grep -E "^##" $(MAKEFILE_LIST) | sed 's/^## /  /'
