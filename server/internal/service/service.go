package service

import (
	"fabrlyn.com/smultron/server/internal/store"
	"fabrlyn.com/smultron/server/internal/model"
	"github.com/jackc/pgx/v5"
)

func CreateHub(conn *pgx.Conn, createHub model.CreateHub) (model.Hub, error) {
  return store.CreateHub(conn, createHub)
}

func ListHubs(conn *pgx.Conn) ([]model.Hub, error) {
  return store.ListHubs(conn)
}
