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

## assembly


### onshape
[cad project](https://cad.onshape.com/documents/0dfa4ee4d09bd93c2bde7585/w/f97345cb2106d2a261ad7e17/e/78e917287c298807d1374525?renderMode=0&uiState=650ff2563e0e525b6c274b7b)

### instructions

Glue the mount to the case.

Insert the device and slide on the lid.

![front](https://drive.google.com/uc?id=1owRORHNZ7k9dWp0mdpQ7rrQz7TY7Gl8q)
![mount](https://drive.google.com/uc?id=1Pvh6fT73TNqiagfZO5_R0xJxPFTFdCwL)
![case](https://drive.google.com/uc?id=1ol5IyqaGq1Yv376OKaLk9uhQuyTwHsqT)


[^1]: may use custom resources but will have the intention to use regular ipso when suitable

