package store

import (
	"context"
	"errors"
	"fmt"
	"os"
	"time"

	"fabrlyn.com/smultron/server/internal/model"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
)

func CreateConnection() *pgxpool.Pool {
	conn, err := pgxpool.New(context.Background(), os.Getenv("DATABASE_URL"))
	if err != nil {
		fmt.Fprintf(os.Stderr, "Unable to connect to database: %v\n", err)
		os.Exit(1)
	}

	return conn
}

func ListHubs(conn *pgxpool.Pool) ([]model.Hub, error) {
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

    id, err := model.IdFromValue(dbId)
    if err != nil {
      return hubs, err
    }

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

func FindHubById(db *pgxpool.Pool, hubId model.Id) (model.Hub, error) {
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
		&dbUpdatedAt,
		&dbName,
	)

	if err != nil {
		return model.Hub{}, errors.New(fmt.Sprintf("Failed to find hub by id: %+v", err))
	}

	id, err := model.IdFromValue(dbId)
	if err != nil {
	  return model.Hub{}, err
  }

	createdAt := model.TimestampFromValue(dbCreatedAt)
	updatedAt := model.Map(model.NewOption(dbUpdatedAt), model.TimestampFromValue)
	name, err := model.ReferenceFromValue(dbName)
	if err != nil {
		return model.Hub{}, err
	}

	return model.Hub{Id: id, CreatedAt: createdAt, UpdatedAt: updatedAt, Name: name}, nil
}

func CreateHub(db *pgxpool.Pool, id model.Id, request model.CreateHub) (model.Hub, error) {
	_, err := db.Exec(context.Background(), "insert into hub(id, name) values($1, $2)", id.Value(), request.Name.Value())

	if err != nil {
		return model.Hub{}, errors.New(fmt.Sprintf("Failed to create hub: %+v", err))
	}

	return FindHubById(db, id)
}

func CreateThing(db *pgxpool.Pool, createThing model.CreateThing) (model.Thing, error) {
	_, err := db.Exec(
		context.Background(),
		"insert into thing(id, hub_reference, registered_by_hub_id) values ($1, $2, $3)",
		createThing.Id.Value(),
		createThing.HubReference.Value(),
		createThing.RegisteredByHubId.Value(),
	)

	if err != nil {
		return model.Thing{}, err
	}

	return FindThingById(db, createThing.Id)
}

func FindThingById(db *pgxpool.Pool, thingId model.Id) (model.Thing, error) {
	var dbId uuid.UUID
	var dbCreatedAt time.Time
	var dbUpdatedAt *time.Time
	var dbName string
	var dbRegisteredByHubId uuid.UUID

	err := db.QueryRow(
		context.Background(),
		`
		select 
		  id, 
		  created_at, 
		  updated_at, 
		  name,
		  registered_by_hub_id
		from thing 
		where id = $1
		`,
		thingId.Value(),
	).Scan(
		&dbId,
		&dbCreatedAt,
		&dbUpdatedAt,
		&dbName,
		&dbRegisteredByHubId,
	)

	if err != nil {
		return model.Thing{}, errors.New(fmt.Sprintf("Failed to find hub by id: %+v", err))
	}

  id, err := model.IdFromValue(dbId)
  if err != nil {
    return model.Thing{}, err
  }

  registeredByHubId, err := model.IdFromValue(dbRegisteredByHubId)
  if err != nil {
    return model.Thing{}, err
  }

  createdAt := model.TimestampFromValue(dbCreatedAt)
  updatedAt := model.Map(model.NewOption(dbUpdatedAt), model.TimestampFromValue)
  hubReference, err := model.ReferenceFromValue(dbName)
  if err != nil {
    return model.Thing{}, err
  }

	return model.Thing{
		Id:                id,
		CreatedAt:         createdAt,
		UpdatedAt:         updatedAt,
		HubReference:      hubReference,
		RegisteredByHubId: registeredByHubId,
	}, nil
}
