package main

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"os"
	"regexp"
	"time"

	"fabrlyn.com/smultron/server/internal/model"
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
