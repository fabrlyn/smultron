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

type SensorDiscovered struct {
	HubReference model.Reference `json:"hubReference"`
}

func (s SensorDiscovered) ToModel(thingId model.Id, sensorId model.Id) model.SensorDiscovered {
	return model.SensorDiscovered{
		Id:            sensorId,
		HubReference:  s.HubReference,
		PartOfThingId: thingId,
	}
}

type ThingDiscovered struct {
	HubReference model.Reference `json:"hubReference"`
}

func (t ThingDiscovered) ToModel(hubId model.Id, thingId model.Id) model.ThingDiscovered {
	return model.ThingDiscovered{
		Id:                thingId,
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
		fmt.Printf("Failed to parse thing discovered hub id, error: %+v, segments: %+v", err, segments)
		return
	}

	thingId, err := model.ExternalIdFromValue(segments[4])
	if err != nil {
		fmt.Printf("Failed to parse thing discovered thing id, error: %+v, segments: %+v", err, segments)
		return
	}

	service.RegisterThing(inbox.conn, thingDiscovered.ToModel(hubId.Id(), thingId.Id()))
}

func (inbox *Inbox) handleSensorDiscovered(msg *nats.Msg) {
	var sensorDiscovered SensorDiscovered

	err := json.Unmarshal(msg.Data, &sensorDiscovered)
	if err != nil {
		fmt.Printf("Failed to parse thing discovered, error: %+v, data: %+v", err, string(msg.Data))
		return
	}

	segments := strings.Split(msg.Subject, ".")
	thingId, err := model.ExternalIdFromValue(segments[4])
	if err != nil {
		fmt.Printf("Failed to parse sensor discovered thing id , error: %+v, segments: %+v", err, segments)
		return
	}

	sensorId, err := model.ExternalIdFromValue(segments[6])
	if err != nil {
		fmt.Printf("Failed to parse sensor discovered id, error: %+v, segments: %+v", err, segments)
		return
	}

	service.RegisterSensor(inbox.conn, sensorDiscovered.ToModel(thingId.Id(), sensorId.Id()))
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
		case "sensor.discovered":
			inbox.handleSensorDiscovered(msg)
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
