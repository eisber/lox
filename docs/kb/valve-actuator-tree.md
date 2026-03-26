# Valve Actuator Tree

Source: https://www.loxone.com/enen/kb/valve-actuator-tree/

---

The Loxone Valve Actuator Tree is a motorized valve actuator for heating systems. By using common adapter rings, it can be mounted on a wide range of valves, which are typically used for radiant floor heating and radiators.



        [**Datasheet Valve Actuator Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_ValveActuatorTree_100602.pdf)



        [**Datasheet Valve Actuator Tree Gen.1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_ValveActuatorTreeGen1_100225.pdf)



## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [LED states](#led_codes)
- [Device test and calibration run](#test)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)




---


## Mounting


    In delivery state, the valve is fully open. The Valve Actuator Tree performs a calibration run only after powering on, so it must be installed before initial start-up. Before mounting, make sure that the valve pin is movable and not rusted.


    Then snap the actuator onto the adapter ring. Do not apply excessive force. Make sure that the actuator is correctly positioned on the valve. Only use valve adapters suitable for the valve.



| ![warning](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Resources/warning.png) | Ensure that the correct adapter is used for the specific valve type.Using an unsuitable adapter may lead to issues, such as "valve stuck" or "no valve detected" messages. The adapter also positions the valve stem correctly. |
| --- | --- |

    We recommend using the Loxone Tree Cable or Panel Wire, both 0.8mm Ø, to wire the actuators.
Use the IDC tool to push the orange/white wire pair to connect the power supply and the green/white wire pair to connect the Tree data lines.


    We recommend a short service loop between the individual actuators and securing the cable near the first actuator for strain relief.


    The wires are pressed into the terminals of the actuators using the IDC Tool (100226) supplied with the Miniserver or Tree Extension.




![VA Tree Installation](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/VA Tree Installation.png)




| ![warning](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Resources/warning.png) | Make sure that no water can drip onto the actuator.Although the actuator can be mounted in any position, it is recommended to install the actuator above the valve to prevent water dripping onto the actuator in case of a leaking valve or condensation. |
| --- | --- |


| ![warning](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Resources/warning.png) | When installing the Valve Actuator, ensure that the opening for the green mounting clip does not face downward. Installing the Valve Actuator in this position may cause the clip to loosen or fall out, potentially leading to malfunction or improper operation of the valve. |
| --- | --- |


| ![info](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Resources/info.png) | If a large number of valve actuators share one electrical line with a small wire cross-section, significant voltage drop may occur. In such a case, it is recommended to use a larger wire cross-section for the 24V supply line and to switch to a smaller cross-section just before connecting the actuators. |
| --- | --- |

    A separate circuit is recommended for the power supply of the actuators. Note the inrush current of 50mA at 24VDC. In order to avoid load peaks at the moment of switch-on, a maximum of 5 actuators will start moving simultaneously.



---


## Commissioning


    **[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**


    After powering on, the calibration run starts. The Valve Actuator determines the valve travel and registers the 0% and 100% positions.
If necessary, a [Recalibration](#test) can be triggered via Loxone Config.


    By default, the actuator then moves to the 0% position (valve closed). Programming must be created to open the valve again.



---


## LED states


    Please note that the status LEDs do not light up while the actuator is moving.



| LED state | Description |
| --- | --- |
| ![LED VA red flashing no tree](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LED_VA_red_flashing_no_tree.gif) | No communication to Miniserver possible. Please check the wiring. |
| ![LED VA orange flashing not learn](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LED_VA_orange_flashing_not_learn.gif) | Tree Device was detected on Tree bus, but is not paired with Miniserver. |
| ![LED VA 3xgreen flashing](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LED_VA_3xgreen_flashing.gif) | 3 short flashes Communication OK, device paired. |
| ![LED VA red orange green flashing](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LED_VA_red_orange_green_flashing.gif) | Device is in pairing mode, ready for pairing. |
| ![LED VA red green fast flashing](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LED_VA_red_green_fast_flashing.gif) | Device was selected in Loxone Config and is identifying. |
| ![LED VA red flashing](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LED_VA_red_flashing.gif) | Online, valve open more than 90%. |
| ![LED VA orange flashing](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LED_VA_orange_flashing.gif) | Online, valve open between 10% - 90%. |
| ![LED VA green flashing](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LED_VA_green_flashing.gif) | Online, valve less than 10% open. |
| ![LED VA red fast flashing](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LED_VA_red_fast_flashing.gif) | The error "valve stuck" or "no valve detected" is present. Please check the valve pin, it should be easily movable. |


---


## Device test and calibration run


    The actuator can be fully opened or closed for test purposes.
In addition, the actuator can be recalibrated.


    Right-click on the actuator in the Periphery tree and select an option:


    Device test on: Fully open actuator/valve
Device test off: Fully close actuator/valve
Recalibrate device: Perform calibration run.



---


## Sensors




| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| No valve detected | No valve was detected. Input deactivated in Loxone Config. | - | 0/1 |
| Valve is stuck | Indicates a stuck valve. Input deactivated in Loxone Config. | - | 0/1 |
| Input 1 | Digital | 0/1 |








---


## Actuators




| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Valve Actuator | Valve Actuator. | - | - |
| Valve Actuator Tree | % | ∞ |








---


## Diagnostic Inputs




| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Valve Actuator Tree | Indicates whether the device can be reached by the Miniserver.Diagnostics for Air devicesDiagnostics for Tree devicesDiagnostics for Extensions | Digital | 0/1 |
| No Valve Detected | Valve Actuator Tree | Digital | 0/1 |
| Valve is stuck | Valve Actuator Tree | Digital | 0/1 |








---


## Properties




| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Specifies the serial number of the device.Enter 'Auto' to automatically pair a Tree device with unknown serial number.This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup).Save in the Miniserver to pair the Tree device.Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - |
| Switch off status LEDs | If checked, the status LEDs on the device are switched off in normal operation.In the event of a fault, the device will continue to alert you to its status LEDs. | - |








---


## Safety Instructions


    The installation must be carried out by a qualified technician in accordance with all applicable regulations.


    Ensure that the device is protected from water.



---


## Documents



        [**Datasheet Valve Actuator Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_ValveActuatorTree_100602.pdf)



        [**Datasheet Valve Actuator Tree Gen.1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_ValveActuatorTreeGen1_100225.pdf)



---