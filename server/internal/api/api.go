package api

import (
	"encoding/json"
	"fmt"
	"net/http"

	"fabrlyn.com/smultron/server/internal/model"
	"fabrlyn.com/smultron/server/internal/service"
	"github.com/jackc/pgx/v5/pgxpool"
)

type Api struct {
  conn *pgxpool.Pool
}

func NewApi(conn *pgxpool.Pool) Api {
  return Api{conn: conn}
}

func (a *Api) CreateHub(response http.ResponseWriter, request *http.Request) {
  var createHub model.CreateHub

  err := json.NewDecoder(request.Body).Decode(&createHub)
  if err != nil {
    response.WriteHeader(400)
    response.Write([]byte(err.Error()))
    return
  }

  fmt.Printf("Create hub: %+v", createHub)
  hub, err := service.CreateHub(a.conn, createHub)
  if err != nil {
  response.WriteHeader(400)
  response.Write([]byte(err.Error()))
    return 
  }

  body, err := json.Marshal(hub)
  if err != nil {
  response.WriteHeader(500)
  response.Write([]byte(err.Error()))
    return 
  }

  response.WriteHeader(200)
  response.Header().Set("Content-Type", "application/json")
  response.Write(body)
}

func (a *Api) ListHubs(response http.ResponseWriter, request *http.Request) {
  hubs, err := service.ListHubs(a.conn)
  if err != nil {
    response.WriteHeader(500)
    response.Write([]byte(err.Error()))
    return
  }

  body, err := json.Marshal(hubs)
  if err != nil {
    response.WriteHeader(500)
    response.Write([]byte(err.Error()))
    return
  }

  response.WriteHeader(200)
  response.Write(body)
} 

func (a *Api) Run() {
  mux := http.NewServeMux()
  
  mux.HandleFunc("POST /api/hub", a.CreateHub)
  mux.HandleFunc("GET /api/hub", a.ListHubs)
  
  server := http.Server{
    Addr: ":4551",
    Handler: mux,
  }
  server.ListenAndServe()
}
