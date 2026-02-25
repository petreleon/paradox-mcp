# Paradox MCP Makefile

IMAGE_NAME = paradox-mcp
DATA_DIR = $(pwd)/data

.PHONY: all build test run clean help

all: build test

## build: Build the Docker image
build:
	docker build -t $(IMAGE_NAME) .

## test: Run automated tests inside a Docker container
test: build
	docker run -t --rm \
		--entrypoint sh \
		-v $(shell pwd)/tests:/app/tests \
		$(IMAGE_NAME) \
		-c "apt-get update >/dev/null && apt-get install -y python3 >/dev/null && python3 /app/tests/test_mcp.py"

## run: Run the server in interactive mode (requires DATA_DIR)
run:
	@mkdir -p $(DATA_DIR)
	docker run -i --rm \
		-v $(DATA_DIR):/data \
		$(IMAGE_NAME) --location /data --permit-editing

## clean: Remove build artifacts and temporary files
clean:
	cargo clean
	rm -rf data/*.db data/*.PX
	docker rmi $(IMAGE_NAME) || true

## help: Show this help message
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^##' Makefile | sed -e 's/## //' | column -t -s ':'
