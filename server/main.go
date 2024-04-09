package main

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"os"
	"regexp"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func main() {
	// urlExample := "postgres://username:password@localhost:5432/database_name"
	conn, err := pgx.Connect(context.Background(), os.Getenv("DATABASE_URL"))
	if err != nil {
		fmt.Fprintf(os.Stderr, "Unable to connect to database: %v\n", err)
		os.Exit(1)
	}
	defer conn.Close(context.Background())

	reference, err := newReference("hub-abc")
	hub, err := createHub(conn, CreateHub{name: reference})

	fmt.Printf("%v\n", hub)
}

type Reference struct {
	value string
}

func newReference(value string) (Reference, error) {
	value_regexp, e := regexp.Compile("^[a-z]([-_]?[a-z0-9]+)*[a-z0-9]?$")
	if e != nil {
		panic(e)
	}

	if !value_regexp.MatchString(value) {
		return Reference{}, errors.New("Invalid reference")
	}

	return Reference{value}, nil
}

type CreateHub struct {
	name Reference
}

type Hub struct {
	id         uuid.UUID    // TODO: ExternalId
	created_at time.Time    // TODO: Timestamp
	updated_at sql.NullTime // TODO: Maybe[Timestamp]
	name       Reference
}

func findHubById(db *pgx.Conn, id uint64) (Hub, error) {
	var hub Hub
	err := db.QueryRow(context.Background(), "select external_id, created_at, updated_at, name from hub where id = $1", id).Scan(&hub.id, &hub.created_at, &hub.updated_at, &hub.name.value)
	if err != nil {
		return Hub{}, errors.New("Failed to find hub by id")
	}

	return hub, nil
}

func createHub(db *pgx.Conn, request CreateHub) (Hub, error) {
	var id uint64
	err := db.QueryRow(context.Background(), "insert into hub(name) values($1) returning id", request.name.value).Scan(&id)

	if err != nil {
		return Hub{}, errors.New("Failed to create hub")
	}

	return findHubById(db, id)
}
