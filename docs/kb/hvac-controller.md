# HVAC Controller

Source: https://www.loxone.com/enen/kb/hvac-controller/

---

Controller for various types of HVAC systems. This block is optimized to work with at least one Intelligent Room Controller and its intelligent mode switching to control the heating/cooling system.


    Rooms with presence are prioritized for the heating/cooling decision. If presence is detected in rooms that call for heat and in rooms that call for cooling, the decision is based on the higher demand.


    Intelligent Room Controllers provide dedicated outputs for up to three assigned controllers.


    This function block belongs to the family of thermal energy sources. It is also suitable for use with hydronic heating systems.


    This block is incredibly sophisticated and far superior to energy source controllers available on the market, e.g. luxurious thermostats.



| ![warning](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Resources/warning.png) | The HVAC Controller is primarily designed for North American HVAC systems. However, it is not limited to this application and can also be used with a wide range of other system types, depending on the project requirements. |
| --- | --- |


## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)




---


## Inputs




| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Mode | Mode | 0 = off1 = automatic mode, switches automatically between heating and cooling, based on demand2 = heating only3 = cooling only | - | 0...3 |
| ϑo | Outdoor temperature | If this input is not connected, the value of the system variable "Outdoor temperature" is used. | ° | ∞ |
| B | Boost | Activate second stage heating/cooling if the first stage is already active. | - | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off.On: Block is locked.Dominating input.The name of the connected sensor is used in the user interface. | - | 0/1 |
| Emh | Emergency Heat | Activates output (E) when heating demand = 1. Used for emergency heat, e.g. if heat pump defective. | - | 0/1 |
| Fan | Fan | Activates output (G) and opens all Intelligent Room Controllers to 100%.If this input is used, the fan can no longer be activated in the App. | - | 0/1 |
| H | Humidity | Required when output (Hmd) is connected in order to optimize indoor humidity. Note: Use the humidity reading of the most relevant room or an average value. | - | ∞ |
| Ec | Excess cooling | Surplus or cheap cooling energy availableIn cooling mode, Intelligent Room Controllers will overcool or allow premature start of cooling. | - | 0/1 |
| Eh | Excess heating | Surplus or cheap heating energy available.In heating mode, Intelligent Room Controllers will overheat or premature start of heating will be permitted. | - | 0/1 |








---


## Outputs




| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| W/W1 | 1st stage heating | 1st stage heating. Varies depending on selected heating type.For "Oil/gas/electric": active for first stage heat, active for second stage heat.For "Heat pump with fossil fuel backup": active for second stage heat. For "Heat pump with electric backup": active for second stage heat. | 0/1 |
| W2 | 2nd stage heating | 2nd stage heating. Varies depending on selected heating type.For "Oil/gas/electric": active for second stage heat.For "Heat pump with fossil fuel backup": active for second stage heat.For "Heat pump with electric backup": active for second stage heat. | 0/1 |
| Y | Compressor | Compressor. Varies depending on selected heating type.For "Oil/gas/electric": active for first stage cool, active for second stage cool.For "Heat pump with fossil fuel backup": active for first stage heat/cool, active for second stage cool.For "Heat pump with electric backup": active for first stage heat/cool, active for second stage heat/cool. | 0/1 |
| Y2 | 2nd stage cooling | 2nd stage cooling. Varies depending on selected heating type.For "Oil/gas/electric": active for second stage cool.For "Heat pump with fossil fuel backup": active for second stage cool.For "Heat pump with electric backup": active for second stage heat/cool. | 0/1 |
| E | Emergency Heat | Activated depending on input (Eh) or via app activation. | 0/1 |
| O/B | Reversing valve | Changes the operating direction of the heat pump. If (DirV) is deactivated: OFF for heating, ON for cooling. If (DirV) is activated: ON for heating, OFF for cooling. | 0/1 |
| G | Fan | Fan that moves heated or cooled air into the rooms | 0/1 |
| Hmd | Humidifier | Activated in heating mode if (H) < (Hs) | 0/1 |
| API | API Connector | Intelligent API based connector.API Commands | - |








---


## Parameters




| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| minO | Minimum OFF time | Heating and cooling must be off for longer than (minO) before reactivating. Note: This prevents frequent switching on and off of the heating/cooling system and thus extends the system life. | s | 0...∞ | 300 |
| Sot | Switch-on threshold | Switch-on threshold for heating/cooling. The arithmetic mean of the demand (degree of opening * room size) of all Intelligent Room Controllers must be greater than (Sot) to activate heating/cooling. If a room with motion is calling for heating/cooling, all Intelligent Room Controllers that are in Eco mode will be opened after 15 minutes.WARNING: Defines the minimum threshold to prevent overheating or freezing of the system | % | 0...100 | 30 |
| Fot | Fan overrun time | Fan overrun time after heating or cooling. Transports residual energy from the system to the rooms. | s | 0...∞ | 90 |
| Tt2s | Time to second stage | If heating/cooling is active for longer than the set time, the second stage heating/cooling is activated. | s | 0...∞ | 300 |
| ∆ϑ | Delta temperature second stage | If the difference between the setpoint and actual temperature of an Intelligent Room Controller exceeds the set value, the second heating/cooling stage is activated. It is deactivated again when the difference reaches 0°. | ° | 0...∞ | 2 |
| mioϑc | Minimum outdoor temperature cooling | If used outdoor temperature (Parameter Otm) falls below this value only heating is allowed.This parameter is only visible in certain configurations. | ° | ∞ | 15 |
| maoϑh | Maximum outdoor temperature heating | If used outdoor temperature (Parameter Otm) exceeds this value only cooling is allowed.This parameter is only visible in certain configurations. | ° | ∞ | 18 |
| ϑpmic | Protection temperature minimum cooling | Minimum outdoor temperature for activating the heat pump in Cooling mode to protect it from damage. | ° | ∞ | 12 |
| ϑpmih | Protection temperature minimum heating | Minimum outdoor temperature for activating the heat pump in heating mode to protect it from damage.Applies only to heating type 'Heat pump with fossil fuel backup'.This parameter is only visible in certain configurations. | ° | ∞ | 0 |
| ϑpmah | Protection temperature maximum heating | Maximum outdoor temperature for activating the heat pump in heating mode.Does not apply to heating type 'Oil/Gas/Electric'.This parameter is only visible in certain configurations. | ° | ∞ | 19 |
| Hs | Humidity setpoint | If the humidity falls below the setpoint in heating mode, the humidifier is activated until the humidity exceeds the setpoint by 2. | % | 0...100 | 45 |
| DirV | Direction valve | Changes the operating mode of the reversing valve. | - | 0/1 | 0 |
| Otm | Outdoor Temperature Mode | 0 = Disabled (mioϑh and mioϑc not used)1 = Average Outdoor Temperature of the past 48h2 = Value of the System Variable 'Expected Average Outdoor Temperature 48h'3 = Current Outdoor TemperatureIf enabled, the average outdoor temperature of the last 48h, next 48h or the current temperature is used to select heating/cooling mode according to (mioϑh) and (mioϑc).If the value is not available, this parameter has no effect. | - | 0...3 | 3 |








---


## Properties




| Summary | Description | Default Value |
| --- | --- | --- |
| Heating Type | Type of heating. The type determines how the outputs are controlled. | - |
| Assign room controllers | Add or remove the Climate Controller as a source for individual Intelligent Room Controllers.Further settings (Priority, PWM, etc.) can be made in the configuration dialog of each Intelligent Room Controller. | - |








---