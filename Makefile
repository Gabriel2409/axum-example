
.PHONY: help run quickdev postgres psql

# Display help message
help:
	@echo "Available targets:"
	@echo "  - help: Display this help message."
	@echo "  - run: Runs the app"
	@echo "  - quick_dev: Runs the quick_dev target for rapid development"
	@echo "  - postgres: Starts postgresql server docker image"
	@echo "  - psql: Access the docker container and run psql command" 

run:
	cargo watch -q -c -w src/ -w .cargo/ -x "run"

quick_dev:
	cargo watch -q -c -x "run --example quick_dev"

postgres:
	docker run --rm --name pg -p 5432:5432 -e POSTGRES_PASSWORD=welcome postgres:15

psql:
	docker exec -it -u postgres pg psql
