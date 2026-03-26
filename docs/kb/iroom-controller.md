# iRoom Controller

Source: https://www.loxone.com/enen/kb/iroom-controller/

---

## GENERAL

Using the iRoom Controller you can integrate your dock products from iRoom into your Loxone home. For further info on iRoom products click [here](http://www.iroomsidock.com/).  The inputs and outputs of iRoom network devices can be integrated via the network interface.

## IROOM CONTROLLER

In order to insert an iRoom controller, first select “Network devices” in the peripheral tree, then the “Add network devices” button appears at the top of the screen in the ribbon. A drop-down menu opens and you can select and insert the iRoom controller.

*[iRoom controller configuration]*

In the properties of iRoom Controller, you can enter the Network Address. The default port for communication is 13601 it will automatically be appended to the entered address so there is no need to add this manually yourself.

*[iRoom Controller Properties Panel]*

*[Icon Exclamation Mark Loxone]*The operation and installation instructions for the iRoom devices can be found [here](http://www.iroomsidock.com/download/).

## SENSORS AND ACTUATORS

The network device iRoom Controller contains a series of sensors and actuators (inputs & outputs) that can be used in the configuration:

#### SENSORS

| Buttons 1-8, and Home Button | Buttons on the front of the wall holder |
| --- | --- |
| Digital Inputs 0-8 | Digital inputs of the wall bracket details, refer to the instructions of iRoom’s iBezel |
| Digital Inputs 0-8 | Is active as soon as an iPad is docked in the posture |
| Relay | Status of relay on bracket |
| Volume | Current volume |
| Proximity Sensor | Proximity sensor of the bracket |

#### ACTUATORS

In the iDock, Relay, Mute, Play / Pause actuators, there are commands for ON, OFF and Toggle (Toggle = Change state)

| Beep | Output the sound signal on the bracket |
| --- | --- |
| IDock (close / open / toggle) | Commands to control the iDock |
| Relay (close / open / toggle) | Commands to control the relay mounted on the wall bracket |
| Mute / Unmute / Toggle Mute | Commands to control the mute state of the mount |
| Play / Pause / Toggle Play / Pause | Command to play / stop media |
| Volume | Analogue volume output |
| Volume down | Audio output |
| Volume up | Audio output |
| Text Command | User-defined text commands can be used by “;” Separately. |