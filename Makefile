
.PHONY: help run quickdev 

# Display help message
help:
	@echo "Available targets:"
	@echo "  - help: Display this help message."
	@echo "  - run: Runs the app"
	@echo "  - quick_dev: Runs the quick_dev target for rapid development"

run:
	cargo watch -q -c -w src/ -w .cargo/ -x "run"

quick_dev:
	cargo watch -q -c -x "run --example quick_dev"
