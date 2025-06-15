.PHONY: dev-shell build test test-integration fmt lint clean

# Docker image name for development environment
DEV_IMAGE := spire-client-dev

# Build development Docker image
.dev-image:
	@if docker buildx version >/dev/null 2>&1; then \
		echo "Using docker buildx..."; \
		docker buildx build --load -f Dockerfile.dev -t $(DEV_IMAGE) . || \
		(docker buildx create --use && docker buildx build --load -f Dockerfile.dev -t $(DEV_IMAGE) .); \
	else \
		echo "Using docker build..."; \
		docker build -f Dockerfile.dev -t $(DEV_IMAGE) .; \
	fi
	@touch .dev-image

# Launch development shell
dev-shell: .dev-image
	docker run --rm -it \
		-v $(PWD):/workspace \
		-w /workspace \
		$(DEV_IMAGE)

# Build the project
build:
	go build ./...

# Run unit tests
test:
	go test ./...

# Run integration tests (requires SPIRE Server to be running)
test-integration:
	INTEGRATION_TEST=true go test ./test/integration/...

# Format code
fmt:
	go fmt ./...

# Run linter
lint:
	golangci-lint run

# Clean build artifacts
clean:
	rm -f .dev-image
	go clean