#include "ESP8266mDNS.h"

MDNSResponder mdnsResponder;

void mdnsLoop()
{
  mdnsResponder.update();
}

void mdnsSetup()
{
  mdnsResponder = MDNSResponder();

  String mdnsName = "alpha_" + WiFi.macAddress();

  while (!mdnsResponder.begin(mdnsName))
  {
      Serial.println("mdns | Failed to setup");
      delay(1000);
  }
  mdnsResponder.addService("coap", "udp", 5683);
  mdnsResponder.addServiceTxt("coap", "udp", "hack-for", "fixing 0 length TXT");
}
