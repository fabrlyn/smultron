package api

import (
	"encoding/json"
	"net/http"

	"fabrlyn.com/smultron/server/internal/service"
	"github.com/jackc/pgx/v5"
)

type Api struct {
  conn *pgx.Conn
}

func NewApi(conn *pgx.Conn) Api {
  return Api{conn: conn}
}

func (a *Api) CreateHub(response http.ResponseWriter, request *http.Request) {
  response.WriteHeader(200)
  response.Write([]byte("Hej there!"))
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
