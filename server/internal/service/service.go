package service

import (
	"fabrlyn.com/smultron/server/internal/store"
	"fabrlyn.com/smultron/server/internal/model"
	"github.com/jackc/pgx/v5/pgxpool"
)

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
