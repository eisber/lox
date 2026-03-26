# Energy Meter 1-Phase Tree

Source: https://www.loxone.com/enen/kb/energy-meter-1-phase-tree/

---

The Loxone Energy Meter for DIN rail mounting features MID certified bidirectional Energy metering for up to 100A. The measured values can be used for energy management, consumption data collection and much more.



        [**Datasheet Energy Meter 1-Phase Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_EnergyMeter_1PhaseTree_100566.pdf)



## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Wiring Direction](#MeterWiringDirection)
- [Meter Function Block](#MeterBlock)
- [Additional Sensors](#MeterAddSensors)
- [Display](#Display)
- [LED Status](#led_state)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)




---


## Mounting


    The meter is connected as follows:



    The neutral conductor from the grid is either routed through the meter or connected only on the input.
If necessary, the [wiring direction](#MeterWiringDirection) can be changed, and the meter wired from bottom to top.


    After connecting the mains lines, the covers must be placed over the mains terminal blocks, followed by connecting the Tree and 24V lines.



|  | The meter is powered by mains voltage. The +24V terminal is not connected internally but is present to allow for easy daisy-chaining of the +24V line. The GND must be connected for the proper function of the Tree interface. |
| --- | --- |


|  | Do not install power supplies directly next to the Energy Meter, as this can lead to inaccurate measurements. |
| --- | --- |


---


## Commissioning


    For commissioning, the device must be supplied with power and a connection to the Miniserver must be possible via the Tree Interface.


    **[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**



---


## Wiring Direction


    If it is necessary due to the wiring layout, the meter can also be wired from bottom to top.


    In this case, the Wiring Direction setting in Loxone Config must immediately be set to “inverted” to ensure that energy is added at the correct meter readings and the power is displayed with the correct sign.
The meter readings themselves and their inputs are not interchanged.


    The wiring direction setting also changes the sign of the power on the display, but not the meter readings and their arrows on the display.



---


## Meter Function Block


    After pairing the meter, the Meter Type must be selected.


    Subsequently, the meter can be dragged to the programming page. This action automatically creates the Meter Block:



    By default, the API Connector transmits the meter reading and power at the following intervals:
Power: Every 5 seconds, but only on value change.
Meter readings: Every 60 seconds, but only on value change. The transmission cycles can be individually configured in the device settings.


    Alternatively, sensors can be [manually inserted](#MeterAddSensors), and their polling cycle can be set. These are then used at the inputs of the meter block instead of the API Connector.



---


## Additional Sensors


    The meter provides additional measurements through analog sensors that must first be added. Afterward, the desired measurement is selected:



    The number of available sensors depends on the type of meter and the configured Type of use.



---


## Display


    The display alternately shows the following values:



    The wiring direction setting also changes the sign of the power on the display, but not the meter readings and their arrows on the display.



---


## LED Status



| Status LED | Description |
| --- | --- |
|  | Everything OK, device is online. |
|  | Connection to the Miniserver is okay, but the device has not been paired. |
|  | Device cannot connect to the Miniserver via the Tree interface. |
|  | Device was selected in Loxone Config and is identifying. |
|  | Update is in progress. |
|  | No AC mains supply ornormal operation (if Status LED is switched off) |
|  | Internal device communication error |


---


## Actuators




| Summary | Description |
| --- | --- |
| API Connector | Intelligent API based connector.API Commands |








---


## Diagnostic Inputs




| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Energy Meter Tree | Indicates whether the device can be reached by the Miniserver.Diagnostics for Air devicesDiagnostics for Tree devicesDiagnostics for Extensions | Digital | 0/1 |








---


## Properties




| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - |
| Serial Number | Specifies the serial number of the device.Enter 'Auto' to automatically pair a Tree device with unknown serial number.This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup).Save in the Miniserver to pair the Tree device.Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - |
| Meter Type | Choose which type of Meter Function Block you want to insert.Meter: only one energy direction is monitored.Meter Bidirectional: both energy directions are monitored.Meter for Storage: for use with Batteries, bidirectional with charge Level.Wallbox: for use with EV Charging Wallboxes. | - | - |
| Wiring direction | Sets the wiring direction of the meter. | - | - |
| Switch off status LEDs | If checked, the status LEDs on the device are switched off in normal operation.In the event of a fault, the device will continue to alert you to its status LEDs. | - | - |
| Transmission cycle power [s] | Minimum transmission cycle for power values via API actor. Values are updated only when they change. | 0.1...4095 | 5 |
| Transmission cycle energy [s] | Minimum transmission cycle for energy values via API actor. Values are updated only when they change. | 0.1...4095 | 60 |
| Monitor validation | If checked, you will be notified via System Status or Cloud Mailer when the value is out of the set range or at timeout. | - | - |








---


## Safety Instructions


    Installation must be carried out by a qualified electrician in accordance with the applicable regulations.


    This device must be mounted on a DIN rail in an electrical distribution enclosure to ensure protection against contact, water and dust.


    The selection of conductor cross-sections and associated overcurrent protection devices is dictated by national and international standards and installation guidelines. This requires choosing a conductor cross-section suitable for the loads rated current, as well as considering the insulation material of the cable, method of installation, and ambient temperature.



---


## Documents



        [**Datasheet Energy Meter 1-Phase Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_EnergyMeter_1PhaseTree_100566.pdf)



---