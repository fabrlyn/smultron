#include <Arduino.h>
#include <ESP8266WiFi.h>
#include <ESP8266WebServer.h>
#include <DNSServer.h>
#include <WifiManager.h>

bool        wifiConnected;
WiFiManager wifiManager;

void setup()
{
  Serial.begin(115200);

  wifiConnected = wifiManager.autoConnect();
}

void loop()
{
}
