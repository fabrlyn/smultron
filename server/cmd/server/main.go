package main

import (
  "os"
	"context"
	"fmt"

	"fabrlyn.com/smultron/server/internal/model"
	"fabrlyn.com/smultron/server/internal/store"
	"fabrlyn.com/smultron/server/internal/api"
)

func main() {
  connection := store.CreateConnection()
	defer connection.Close(context.Background())

  api := api.NewApi(connection)
  api.Run()

  id, err := model.NewId()
	if err != nil {
    fmt.Errorf("Failed to create id for new hub");
	  os.Exit(1);
  }

	reference, err := model.ReferenceFromValue("hub-def")
	if err != nil {
	  fmt.Errorf("Failed to create reference for new hub");
	  os.Exit(1);
  }

  hub, err := store.CreateHub(connection, model.CreateHub{Id: id, Name: reference })
	fmt.Printf("%v\n", hub)
	fmt.Printf("%v\n", err)
}

