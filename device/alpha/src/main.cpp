#include <Arduino.h>

#include "coap.h"
#include "mdns.h"
#include "wifi.h"

void loop()
{
  coapLoop();

  mdnsLoop();
}

void setup()
{
  Serial.begin(115200);

  wifiSetup();

  coapSetup();

  mdnsSetup();
}
