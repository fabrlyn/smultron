package inbox

import (
	"encoding/json"
	"fmt"
	"strings"

	"fabrlyn.com/smultron/server/internal/model"
	"fabrlyn.com/smultron/server/internal/service"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/nats-io/nats.go"
)

// Topic to hub
// in.hub.<id>
//  .device.<id>

// in.hub.<id>
//  .device.<id>
//    .actuator.<id>
//    .sensor.<id>

// out.hub.<id>
//  .device.<id>
//    .actuator.<id>
//    .sensor.<id>

type Inbox struct {
	conn *pgxpool.Pool
}

func NewInbox(conn *pgxpool.Pool) Inbox {
	return Inbox{
		conn: conn,
	}
}

type ThingDiscovered struct {
	Id           model.Id        `json:"id"`
	HubReference model.Reference `json:"hubReference"`
}

func (t ThingDiscovered) ToModel(hubId model.Id) model.ThingDiscovered {
	return model.ThingDiscovered{
		Id:                t.Id,
		HubReference:      t.HubReference,
		RegisteredByHubId: hubId,
	}
}

func (inbox *Inbox) handleThingDiscovered(msg *nats.Msg) {
	var thingDiscovered ThingDiscovered

	err := json.Unmarshal(msg.Data, &thingDiscovered)
	if err != nil {
		fmt.Printf("Failed to parse thing discovered, error: %+v, data: %+v", err, string(msg.Data))
		return
	}

	segments := strings.Split(msg.Subject, ".")
	hubId, err := model.ExternalIdFromValue(segments[2])
	if err != nil {
		fmt.Printf("Failed to parse thing discovered id, error: %+v, segments: %+v", err, segments)
		return
	}

	service.RegisterThing(inbox.conn, thingDiscovered.ToModel(hubId.Id()))
}

func (inbox *Inbox) Run() error {
	conn, err := nats.Connect(nats.DefaultURL)
	if err != nil {
		panic(fmt.Sprintf("Failed to connect to nats: %+v", err))
	}

  _, err = conn.Subscribe("in.hub.>", func(msg *nats.Msg) {
		messageType := msg.Header.Get("messageType")
		if messageType == "" {
			return
		}

		switch messageType {
		case "thing.discovered":
			inbox.handleThingDiscovered(msg)
			return
		case "sensorDiscovered":
			return
		case "readingRegistered":
			return
		default:
			fmt.Printf("Received message with unknown message type. Headers: %+v Body: %+v", msg.Header, string(msg.Data))
			return
		}
	})

	return err
}
