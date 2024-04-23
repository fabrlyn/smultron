#!/bin/zsh

payload=$(cat <<-END
{
  "id": "018f0c8b-5c7d-71b0-a5f7-2283b54581e2",
  "hubReference": "sensor-0"
}
END
)


nats \
  publish in.hub.2wKwfkNYLrwjX0tvjFqoT.thing.2wMdWc160E3BIvUUBULRW.sensor.2wMgX1I7IIOjX0SVEj0Lq \
  -H messageType:sensor.discovered \
  $payload


