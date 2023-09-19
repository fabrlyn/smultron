#include "Thing.CoAP.h"
#include "Thing.CoAP/Server.h"
#include "Thing.CoAP/ESP/UDPPacketProvider.h"

#include "util.h"

using Thing::CoAP::ContentFormat;
using Thing::CoAP::FunctionalResource;
using Thing::CoAP::Request;
using Thing::CoAP::Status;

Thing::CoAP::Server                 coapServer;
Thing::CoAP::ESP::UDPPacketProvider udpPacketProvider;

// 3/0/5852 -> On Time
FunctionalResource resource_3_0_5852 = FunctionalResource("3/0/5852", ContentFormat::ApplicationOctetStream, true);

void coapLoop()
{
  coapServer.Process();
}

void register_3_0_5852()
{
  resource_3_0_5852.OnGet([](Thing::CoAP::Request &request) {
    byte data[4];
    uint32ToBe((millis() / 1000), data);

    return Thing::CoAP::Status::Content(data, 4);
  });

  coapServer.AddResource(resource_3_0_5852);
}

void coapSetup()
{
  register_3_0_5852();

  coapServer.SetPacketProvider(udpPacketProvider);

  coapServer.Start();
}
