package service

import (
	"fabrlyn.com/smultron/server/internal/store"
	"fabrlyn.com/smultron/server/internal/model"
	"github.com/jackc/pgx/v5/pgxpool"
)

type Service struct {
}

func (s *Service) CreateHub(createHub model.CreateHub) error {
  // model.CreateHub should be model.Message<model.CreateHub>
  // Validate incoming request
  // Start transaction
  // Create CreateHub
  // Derive new message and store to outbox 
  // End transaction
  return nil
}

func CreateHub(conn *pgxpool.Pool, createHub model.CreateHub) (model.Hub, error) {
  id, err := model.NewId()
  if err != nil {
    return model.Hub{}, err
  }

  return store.CreateHub(conn, id, createHub)
}

func ListHubs(conn *pgxpool.Pool) ([]model.Hub, error) {
  return store.ListHubs(conn)
}

func RegisterThing(conn *pgxpool.Pool, thingDiscovered model.ThingDiscovered) {
  createThing := model.CreateThingFromThingDiscovered(thingDiscovered)
  store.CreateThing(conn, createThing)
}

func RegisterSensor(conn *pgxpool.Pool, sensorDiscovered model.SensorDiscovered) {
  createSensor := model.CreateSensorFromSensorDiscovered(sensorDiscovered)
  store.CreateSensor(conn, createSensor)
}

func RegisterBooleanReading(conn *pgxpool.Pool, readingRegistered model.ReadingRegistered[bool]) {
  store.RegisterBooleanReading(conn, readingRegistered)
}

func RegisterI32Reading(conn *pgxpool.Pool, readingRegistered model.ReadingRegistered[int32]) {
  store.RegisterI32Reading(conn, readingRegistered)
}
