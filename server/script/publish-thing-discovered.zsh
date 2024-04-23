#!/bin/zsh

payload=$(cat <<-END
{
  "id": "018f0c1b-32a8-7a18-bc3a-6a4b7655eafe",
  "hubReference": "thing-0"
}
END
)


nats \
  publish in.hub.2wKwfkNYLrwjX0tvjFqoT.thing.2wMdWc160E3BIvUUBULRW \
  -H messageType:thing.discovered \
  $payload


