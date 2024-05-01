package main

import (
	"fmt"
	"sync"

	"fabrlyn.com/smultron/server/internal/api"
	"fabrlyn.com/smultron/server/internal/inbox"
	"fabrlyn.com/smultron/server/internal/model"
	"fabrlyn.com/smultron/server/internal/store"
	"fabrlyn.com/smultron/server/internal/web"
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

	waitGroup := sync.WaitGroup{}
	waitGroup.Add(1)

	inbox := inbox.NewInbox(connection)
	api := api.NewApi(connection)
	web := web.NewWeb(connection)

	inbox.Run()
	go func() {
		api.Run()
		waitGroup.Done()
	}()
	go func() {
		web.Run()
		waitGroup.Done()
	}()
	waitGroup.Wait()
}
