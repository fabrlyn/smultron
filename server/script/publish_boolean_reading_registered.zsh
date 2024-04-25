#!/bin/zsh

payload=$(cat <<-END
{
  "registeredAt": "2024-04-25T16:54:19Z",
  "value": true 
}
END
)


nats \
  publish in.hub.2wKwfkNYLrwjX0tvjFqoT.thing.2wMdWc160E3BIvUUBULRW.sensor.2wMgX1I7IIOjX0SVEj0Lq \
  -H messageType:reading.registered \
  -H valueType:boolean \
  $payload


