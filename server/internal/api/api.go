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
  // Validate request
  // Create a new server.model.Message<server.model.CreateHub>
  // Insert the http-service instance id and correlation id into nats key value
  // Send command
  // Any http-service instance picks up HubCreated event and uses the correlation id to look up the requesting http-service instance
  // Publish the http-service internal nats topic to notify the specific http-service instance that the event occured.
  // The same would go for a non-succesfull response or waiting for multiple events to be published.
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
