package web

import (
	"html/template"
	"net/http"

	"github.com/jackc/pgx/v5/pgxpool"
)

type Web struct {
	conn *pgxpool.Pool
}

func NewWeb(conn *pgxpool.Pool) Web {
	return Web{conn: conn}
}

func (w *Web) Run() {
	mux := http.NewServeMux()

	mux.HandleFunc("GET /", w.serveIndex)

	server := http.Server{
		Addr:    ":4552",
		Handler: mux,
	}
	server.ListenAndServe()
}

type Hub struct {
	Name string
}

type Index struct {
	Hubs []Hub
}

func (w *Web) serveIndex(response http.ResponseWriter, request *http.Request) {

	index := template.Must(template.ParseFiles(
		"index.gohtml", "hubs.gohtml", "hub.gohtml"))
	data := Index{Hubs: []Hub{{Name: "Hub 0"}, {Name: "Hub 1"}}}

	index.Execute(response, data)
}
