# Times

Source: https://www.loxone.com/enen/kb/times/

---

## Application

The Miniserver can calculate various times for you. For example, you can create a program that raises the blinds every day at sunrise.

## Basic Programming

For times to be calculated precisely, enter the location where the system is installed. If you are connected to the internet, the geo-coordinates of this location will be determined automatically after entering the address. The internet connection is not required for the calculation of times.

![EN KB Config Times](https://www.loxone.com/enus/wp-content/uploads/sites/13/2021/08/EN_KB_Config_Times.png)

### **TIME FUNCTIONS**

The Time objects can be found in the Periphery tree under Times.

![EN KB Config Times Overview](https://www.loxone.com/enus/wp-content/uploads/sites/13/2021/08/EN_KB_Config_Times_Overview.png)

### **OVERVIEW**

| Name | Description |
| --- | --- |
| Active Operating Time | This input shows the active operating mode as used in the timer. 0 = Public Holiday, 1 = Vacation, 2 = School Holidays/Day off, 3 = Monday, 4 = Tuesday, … |
| Day | Current day of the month 1 to 31 |
| Daylight | Value on by daylight (sunrise til sunset) |
| Daylight 30min | Value on by daylight (30 minutes after sunrise til 30 minutes before sunset) |
| Days Since 2009 | Number of days that have passed since 1/1/2009 |
| Hour | Current Hour (0 – 23) |
| Minutes | Current Minute (0 – 59) |
| Minutes Past Midnight | Number of minutes from midnight until this moment |
| Month | Current Month (1 – 12) |
| Night | Period between Dusk and Dawn |
| Pulse at Dawn | Pulse at the beginning of civil dawn. End of darkness, sun 6° below the horizon. |
| Pulse at Day Change | Pulse at day change |
| Pulse at Dusk | Pulse at the end of civil dusk. Beginning of darkness, sun 6° below the horizon. |
| Pulse at Hour Change | Pulse at hour change |
| Pulse at Minute Change | Pulse at minute change |
| Pulse at Month Change | Pulse at month change |
| Pulse at Second Change | Pulse at second change |
| Pulse at Sunrise | Pulse at sunrise |
| Pulse at Sunset | Pulse at sunset |
| Pulse at Year Change | Pulse at year change |
| Seconds | Current second (0 – 59) |
| Seconds Since Boot | Number of seconds since last start |
| Start Pulse | Pulse at the start of the program |
| Sun Direction | Sun direction in ° (0 = north, 90 = east, 180 = south, 270 = west) |
| Sun Elevation | Sun elevation in ° |
| Time of Dawn | Time of dawn in minutes from midnight |
| Time of Sunrise | Time of sunrise in minutes from midnight |
| Time of Sunset | Time of sunset in minutes from midnight |
| Unix Timestamp | Current date + time in Unix timestamp format (UTC) |
| Year | Current calendar year |

##

## Programming Examples

#### **RAISE BLINDS AT SUNRISE**

It is often desired that the blinds are automatically raised at sunrise.

To do so, simply use one of the available time objects. In the example, the “Pulse at Sunrise” time object was used and connected to a memory flag. This flag is now connected to all Automatic Blinds blocks that are to be raised at sunrise (input Cu).

At sunrise, the pulse is transmitted to the Automatic Blinds blocks via the flag and the blind executes a complete upward movement.

The same function was also implemented in the evening, with the “Pulse at Dusk” time object.

![EN KB Config Times Automatic Blinds](https://www.loxone.com/enus/wp-content/uploads/sites/13/2021/08/EN_KB_Config_Times_Automatic_Blinds.png)

#### **MOTION SENSOR ONLY ACTIVATES LIGHT WHEN THERE IS NO DAYLIGHT**

The light should turn on only when there is no daylight. This can be achieved without any additional costs.

To do so, simply use the “Lighting Controller” function block in combination with the “Daylight” time object.

Connect the motion sensor to the Mv input and the “Daylight” time object to the DisP input.

If daylight is present, the motion sensor is disabled and does not trigger a lighting scene. More detailed information on the Lighting Controller block can be found [here](https://www.loxone.com/enus/kb/lighting-controller/).

![EN KB Config Times Automatic Lights](https://www.loxone.com/enus/wp-content/uploads/sites/13/2021/08/EN_KB_Config_Times_Automatic_Lights.png)

#### **OUTDOOR LIGHTING ON UNTIL MIDNIGHT**

The outdoor lighting should switch on 90 minutes after sunset and switch off at midnight. In addition, the light should also be controllable from the user interface.

To do so, we add a constant with the value 90 to the “Time of Sunset” object. We compare the result of this addition with the “Minutes Past Midnight” time object.

If the two values of “Minutes Past Midnight” and “Time of Sunset + constant 90” are equal, the value 1 is present at input On of the Switch On/Off function block and output Q switches on.

At midnight, the outdoor lighting is to be switched off again. For this, the time object “Minutes Past Midnight” is compared with a constant with the value 0. We connect the output Q of the Equal block with the input Off of the Switch (On/Off) block. If the time object “Minutes Past Midnight” has the value 0, the outdoor lighting is switched off again.

You can adjust the constants to adapt the behavior of this function to your requirements.

![EN KB Config Times Automatic Lights Outside](https://www.loxone.com/enus/wp-content/uploads/sites/13/2021/08/EN_KB_Config_Times_Automatic_Lights_Outside.png)