package main

import (
	"fmt"

	"fabrlyn.com/smultron/server/internal/api"
	"fabrlyn.com/smultron/server/internal/inbox"
	"fabrlyn.com/smultron/server/internal/model"
	"fabrlyn.com/smultron/server/internal/store"
)

func main() {
  thingId, _ := model.NewId()
  externalThingId := thingId.ToExternal()
	fmt.Printf("Thing id: %+v\n", externalThingId.Id().Value().String())
	fmt.Printf("Thing External id: %+v\n", externalThingId.Value())

	id, _ := model.IdFromString("018efcd3-8f73-7d14-9fb3-e053be4b0201")
	externalId := id.ToExternal()
  fmt.Printf("External id: %+v\n", externalId.Value())

	connection := store.CreateConnection()
	defer connection.Close()

	inbox := inbox.NewInbox(connection)
	api := api.NewApi(connection)

	inbox.Run()
	api.Run()
}
