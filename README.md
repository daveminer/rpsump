# rpsump

Turn your Raspberry Pi into a sump pump!

## Overview

This particular sump pump was designed to prevent lawn damage due to moisture output from residential heaters and A/C units. The water from the appliances is routed to the sump where the Pi monitors the water level sensors and operates the pump when needed.

Later stages of this application will output the reclaimed water to a reservoir where it will be used for gardening; the Pi will also control the watering schedule.

## Components

##### Board

This struct collects the hardware interfaces and is intended to be a singleton for the lifetime of the program. Other threads can read the state of inputs or change outputs via synchronous access (Mutex).

##### Input pins

Are configured with a callback that triggers when the state changes between high and low. These callbacks send messages that aggregate in the consumer of an mpsc channel for processing.

##### Output pins

Controlled by the mpsc consumer, the state of output pins is computed.

##### Sump

Group of inputs/callback handlers and outputs that form the sump pump functionality.

![Sump pump diagram](./assets/rp_sump.png)

## Hardware

##### Raspberry Pi

- RPi 3 Model B

- 12v Relay

![Raspberry Pi and 12V Relay Wiring](https://drive.google.com/uc?id=1UQZAugLhoaG8qODDQBWJ980w4ulJBQFf)

##### Pump Reservoir

- 4in. sewer pipe assembly from retail home improvement store
- standard pvc cement

![Pump reservoir](https://drive.google.com/uc?id=1n1YzGied9_GeD2SX95VH9Bm8LnP7bPMG)


##### Sensor and pump assembly


- 5v float switches
- aquarium pump
- flexible pvc
- hobby-grade acrylic sheet
- zip ties

![Sensor and pump assembly](https://drive.google.com/uc?id=1mZDRnuOX3855pdJ-EjUzNaiFuBW8YkLJ)
