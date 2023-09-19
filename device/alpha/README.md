# alpha

A simple IoT device, based on nodemcu v2, for my home 🏡.

The device will simply report it's uptime, but does so with the help of a set of features which provide a foundation for building my home IoT devices.

The purpose of this project is to serve as a template.

## device features

- wifi
- mdns
- coap
- ipso[^1]

## development

### prerequisites

- pio
- nodemcu v2

### building

```
pio run
```

### flashing

```
pio run --target upload
```

### configure

When flashing the device for the first time, or changing a router, the device will enter AP mode.

- Connect to the AP, look for an AP called `alpha_<mac_address>`.
- Visit `192.168.4.1` and a `WiFiManager` webpage should pop up.
- Enter your credentials.


[^1]: may use custom resources but will have the intention to use regular ipso when suitable

