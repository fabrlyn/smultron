all: test

up:
	docker compose up -d 

down:
	docker compose down

migrate: clear
	docker run --rm --network host -v $(PWD)/internal/postgres/migration:/flyway/sql flyway/flyway -url=jdbc:postgresql://localhost:5432/smultron -user=smultron -password=password migrate

test: migrate
	go test -v ./...

clear: up
	PGPASSWORD=password psql -U smultron -d postgres -h localhost -p 5432 -c "drop database smultron"
	PGPASSWORD=password psql -U smultron -d postgres -h localhost -p 5432 -c "create database smultron"

pgcli:
	pgcli -h localhost -p 5432 -u smultron


