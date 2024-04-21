package main

import (
	"fabrlyn.com/smultron/server/internal/store"
	"fabrlyn.com/smultron/server/internal/api"
)

func main() {
  connection := store.CreateConnection()
	defer connection.Close()

  api := api.NewApi(connection)
  api.Run()
}

