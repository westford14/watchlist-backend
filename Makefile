lint:
	cargo fmt --all
.PHONY: lint

test:
	cargo test
.PHONY: test

stop-server:
	docker compose -f docker-compose.yaml down
.PHONY: stop-server

start-server: stop-server
	docker compose -f docker-compose.yaml up --build
.PHONY: start-server

stop-full:
	docker compose -f docker-compose-full.yaml down
.PHONY: stop-full

start-full: start-server
	docker compose -f docker-compose-full.yaml up --build
.PHONY: start-full

clean-docker: stop-server
	docker system prune -f
	docker images | grep 'watchlist-backend' | awk '{print $$3}' | xargs docker rmi
.PHONY: clean-docker