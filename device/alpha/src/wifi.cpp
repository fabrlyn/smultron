#include <DNSServer.h>
#include <ESP8266WebServer.h>
#include <ESP8266WiFi.h>
#include <WiFiManager.h> 

WiFiManager wifiManager;

void wifiSetup()
{
  wifiManager.autoConnect(("alpha_" + WiFi.macAddress()).c_str());
}

