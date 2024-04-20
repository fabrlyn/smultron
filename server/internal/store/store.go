package store

import (
	"context"
	"errors"
	"fmt"
	"os"
	"time"

	"fabrlyn.com/smultron/server/internal/model"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func CreateConnection() *pgx.Conn {
	conn, err := pgx.Connect(context.Background(), os.Getenv("DATABASE_URL"))
	if err != nil {
		fmt.Fprintf(os.Stderr, "Unable to connect to database: %v\n", err)
		os.Exit(1)
	}

	return conn
}

func ListHubs(conn *pgx.Conn) ([]model.Hub, error) {
	rows, err := conn.Query(context.Background(), "select id, created_at, updated_at, name from hub order by name asc")
	if err != nil {
		return []model.Hub{}, err
	}

	hubs := make([]model.Hub, 0)

	for rows.Next() {
		var dbId uuid.UUID
		var dbCreatedAt time.Time
		var dbUpdatedAt *time.Time
		var dbName string

		err := rows.Scan(
			&dbId,
			&dbCreatedAt,
			&dbUpdatedAt,
			&dbName,
		)
		if err != nil {
			return hubs, err
		}

		id := model.IdFromValue(dbId)
		createdAt := model.TimestampFromValue(dbCreatedAt)
		updatedAt := model.Map(model.NewOption(dbUpdatedAt), model.TimestampFromValue)
		name, err := model.ReferenceFromValue(dbName)
		if err != nil {
			return hubs, err
		}

		hubs = append(hubs, model.Hub{Id: id, CreatedAt: createdAt, UpdatedAt: updatedAt, Name: name})
	}

	return hubs, nil
}

func FindHubById(db *pgx.Conn, hubId model.Id) (model.Hub, error) {
	var dbId uuid.UUID
	var dbCreatedAt time.Time
	var dbUpdatedAt *time.Time
	var dbName string

	err := db.QueryRow(
		context.Background(),
		"select id, created_at, updated_at, name from hub where id = $1",
		hubId.Value(),
	).Scan(
		&dbId,
		&dbCreatedAt,
		dbUpdatedAt,
		&dbName,
	)

	if err != nil {
		return model.Hub{}, errors.New("Failed to find hub by id")
	}

	id := model.IdFromValue(dbId)
	createdAt := model.TimestampFromValue(dbCreatedAt)
	updatedAt := model.Map(model.NewOption(dbUpdatedAt), model.TimestampFromValue)
	name, err := model.ReferenceFromValue(dbName)
	if err != nil {
		return model.Hub{}, err
	}

	return model.Hub{Id: id, CreatedAt: createdAt, UpdatedAt: updatedAt, Name: name}, nil
}

func CreateHub(db *pgx.Conn, request model.CreateHub) (model.Hub, error) {
	err := db.QueryRow(context.Background(), "insert into hub(id, name) values($1, $2)", request.Id.Value(), request.Name.Value())

	if err != nil {
		return model.Hub{}, errors.New(fmt.Sprintf("Failed to create hub: %v", err))
	}

	return FindHubById(db, request.Id)
}
