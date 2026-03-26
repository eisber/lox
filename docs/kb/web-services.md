# Web Services

Source: https://www.loxone.com/enen/kb/web-services/

---

Using various Web Service commands, you can retrieve information from the [Miniserver](https://www.loxone.com/enen/kb/miniserver/) via a browser’s address bar, adjust settings, or switch outputs.

[More detailed information about the API](https://www.loxone.com/enen/kb/api//)

## GENERAL
- Web service commands support both internal and external access.
- The current Miniserver also supports HTTPS.

The structure of a Web Service command is as follows:

http://user:password@IPAddressMiniserver/command/control/value

					When using HTTP, the access credentials in the URL are transmitted in plain text!
For security reasons, when using HTTP (Miniserver Gen.1), it is recommended to omit the access credentials when entering a command in the address bar. Instead, the browser will prompt you for login details in a separate window.

[
![commandlogin](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/commandlogin.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/commandlogin.png)

Some commands require a [user with full access](https://www.loxone.com/enen/kb/user-and-rights-management/).

Once the command is executed, the result is displayed in the browser.

For example, the following command retrieves the current software version of the Miniserver:

[
![browserversioncommand](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/browserversioncommand.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/browserversioncommand.png)

If the command is successful, the response will include the status code **“200”**, indicating that the request was accepted and processed.

### Structure in detail
- control: can be any input or output, real or virtual. Outputs can only be used for status requests.
- value: when ‘value’ is not available, the current control value is returned.

The following values are allowed for digital values:

“**Ein**“, “**On**“, “**Aus**“, “**Off**“, “**1**“, “**0**“, “**Impuls**” or “**Pulse**“.
- Virtual digital inputs with 2 outputs (up-down, left-right, starting from version 1.5.4.16), the following values are allowed.

| ImpulsPlus ImpulsMinus | PulseUp PulseDown | ImpulsAuf ImpulsAb | PulseOpen PulseClose |
| --- | --- | --- | --- |
| PlusEin PlusAus | UpOn UpOff | AufEin AufAus | OpenOn OpenOff |
| MinusEin MinusAus | DownOn DownOff | AbEin AbAus | CloseOff CloseOff |

**The following values are allowed for analog inputs:**

(0-10V) inputs operate within a range of **0** to **10.00**.
- There is no formal limit (e.g., **temperatures** of **21.5** or **-5.2** are valid).
- A dot (`.`) is used as the decimal separator.
- **The following values are allowed for text inputs:**

The **Miniserver** operates using **UTF-8**, so any character supported by UTF-8 should work correctly.

#### Example – Pulse Input

Before sending a pulse into the system, your Miniserver needs a Virtual Input that will accept the command.

[
![Test virtual input](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Test-virtual-input.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Test-virtual-input.png)

Now once we have our Virtual Input added and saved into the Miniserver, the command will need to be sent. Below is an example of a 3rd party application* sending a pulse command into the Loxone Miniserver:

![EN_KB_Computer_Postman_HTT_Command](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Screen-Shot-2017-03-08-at-08.42.24.png)

In this example we have the default credentials (username: admin, password: admin) and the virtual input in the command which has the name ‘Test Input’. By pressing the ‘Send’ button in the application, the string will use the IP address to find the Miniserver on the network, login and then process the command.

If the send command is accepted, the returned response in an XML format will be:

![EN_KB_Computer_Postman_HTT_Command_XML_Response](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Screen-Shot-2017-03-08-at-09.40.12.png)

You will also see in Loxone Config the Virtual Input light up for a split second if ‘Live View’ is active:
![EN_KB_Config_Virtual_Input_On](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Screen-Shot-2017-03-08-at-09.26.10-1.png)

**In these examples, Postman is the 3rd party app being used to send the commands.*

## STATUS & CONTROL

#### QUERY: STATUS OF A CONTROL

| Command | http://miniserver/dev/sps/io/LivingroomLight/state |
| --- | --- |
| Function | Returns the status at the input |
| Answer | <LL control=”dev/sps/io/LivingroomLight” value=”0″ Code=”200″/> |
| Command | http://miniserver/dev/sps/io/LivingroomLight/astate |
| Function | Returns the value at the requested outputs (either a unique value for all – or a ? for different values. |
| Answer | <LL control=”dev/sps/io/LivingroomLight” value=”1″ Code=”200″/> |
| Command | http://miniserver/dev/sps/io/LivingroomLight/all |
| Function | Returns the number of requested outputs and a list of the values at those outputs |
| Answer | <LL control=”dev/sps/io/LivingroomLight/all” value=”8″ Code=”200″ n1=”Aktor (Relais) Q1″ u1=”68347a0e-3bfc-11df-85f6ea34eeb9b08a” v1=”0″ n2=”Aktor (Relais) Q2″ u2=”68347a0e-3bfc-11df-85f6ea34eeb9b08a” v2=”0″ n3=”Aktor (Relais) Q3″ u3=”68347a0e-3bfc-11df-85f6ea34eeb9b08a” v3=”0″ n4=”Aktor (Relais) Q4″ u4=”68347a0e-3bfc-11df-85f6ea34eeb9b08a” v4=”0″ n5=”Aktor (Relais) Q5″ u5=”68347a0e-3bfc-11df-85f6ea34eeb9b08a” v5=”0″ n6=”Aktor (Relais) Q6″ u6=”68347a0e-3bfc-11df-85f6ea34eeb9b08a” v6=”1″ n7=”Aktor (Relais) Q7″ u7=”68347a0e-3bfc-11df-85f6ea34eeb9b08a” v7=”0″ n8=”Aktor (Relais) Q8″ u8=”68347a0e-3bfc-11df-85f6ea34eeb9b08a” v8=”0″/> |
| Command | http://miniserver/status |
| Function | Lists all the devices that are connected to the Miniserver, along with status |
| Answer | <Status Modified=” “> <Miniserver Name=”LoxMiniserver” Internalname=” ” IP=” ” Mask=” ” Gateway=” ” DHCP=” ” DNS1=” ” DNS2=” ” MAC=” ” Device=” ” Version=” “><Extension Name=” ” Internalname=” ” Serial=” ” Version=” ” Online=” “/></Miniserver></Status> |

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Status requests via web service are only possible with inputs and outputs and are not possible with function blocks.

#### PUSH BUTTON

| Command | http://miniserver/dev/sps/io/PushbuttonLivingRoomLight/On |
| --- | --- |
| Function | Control the analogue or digital input |
| Answer | <LL control=”dev/sps/io/PushbuttonLivingRoomLight” value=”1″ Code=”200″/> |

#### Example – Push Button

Before sending a command into the system, your Miniserver needs a Push Button function block that will accept the command.

[
![Pushbutton](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Pushbutton.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Pushbutton.png)

Now once we have our Push Button added and saved into the Miniserver, the command will need to be sent. Below is an example of a 3rd party application* sending an ‘On’ command into the Loxone Miniserver:

![EN_KB_Computer_Postman_HTTP_Command](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Screen-Shot-2017-03-08-at-10.01.45.png)

In this example we have the default credentials (username: admin, password: admin) and the Push Button in the command which has the name ‘Push-Button’. By pressing the ‘Send’ button in the application, the string will use the IP address to find the Miniserver on the network, login and then process the command. We can change the last part of the command to be ‘On’ or ‘Off’, depending on different situations.

If the send command is accepted, the returned response in an XML format will be:

![EN_KB_Computer_Postman_HTT_Command_XML_Response](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Screen-Shot-2017-03-08-at-10.17.58.png)

Code “200” means the command was accepted and processed.

You will also see in the Loxone Interface the Push Button will change state depending on which command is sent (on/off).

[
![pushbuttonapp](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/pushbuttonapp.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/pushbuttonapp.png)

**In these examples, Postman is the 3rd party app being used to send the commands.*

#### QUERY: STATUS CHANGES

| Command | http://miniserver/dev/sps/enablestatusupdate |
| --- | --- |
| Function | Enables automatic sending of status changes in WebSocket. |
| Answer | Sends the current status via WebSocket |

## Miniserver Configuration, Statistics & Commands

The following commands provide **system control, PLC operations, network statistics, and configuration options** for the Miniserver.

### 1. PLC Commands (Programmable Logic Controller)

These commands allow you to monitor and control the PLC system.

| Command | Function |
| --- | --- |
| http://<miniserver-ip>/dev/sps/state | Retrieve PLC status (0-8 states) 0 – No status 1 – PLC booting 2 – PLC program is loaded 3 – PLC has started 4 – Loxone Link has started 5 – PLC running 6 – PLC change 7 – PLC error 8 – Update is occuring |
| http://<miniserver-ip>/dev/sps/status | Retrieve current PLC frequency |
| http://<miniserver-ip>/dev/sps/restart | Restart the PLC |
| http://<miniserver-ip>/dev/sps/stop | Stop the PLC |
| http://<miniserver-ip>/dev/sps/run | Resume PLC execution |
| http://<miniserver-ip>/dev/sps/log/ip | Enable logging to a specific IP |
| http://<miniserver-ip>/dev/sps/log | Stop logging |
| http://<miniserver-ip>/dev/sps/enumdev | List all PLC devices (Miniserver & extensions) |
| http://<miniserver-ip>/dev/sps/enumin | List all PLC inputs |
| http://<miniserver-ip>/dev/sps/enumout | List all PLC outputs |
| http://<miniserver-ip>/dev/sps/identify | Identify the Miniserver or extensions (Requires serial number) |

---

### 2. Configuration Commands

These commands allow you to retrieve or modify Miniserver settings.

| Command | Function |
| --- | --- |
| http://<miniserver-ip>/dev/cfg/mac | Retrieve MAC address |
| http://<miniserver-ip>/dev/cfg/version | Retrieve firmware version |
| http://<miniserver-ip>/dev/cfg/versiondate | Retrieve firmware production date |
| http://<miniserver-ip>/dev/cfg/dhcp | Retrieve or set DHCP configuration |
| http://<miniserver-ip>/dev/cfg/ip | Retrieve or set IP address |
| http://<miniserver-ip>/dev/cfg/mask | Retrieve or set IP mask |
| http://<miniserver-ip>/dev/cfg/gateway | Retrieve or set Gateway address |
| http://<miniserver-ip>/dev/cfg/device | Retrieve or set Miniserver device name |
| http://<miniserver-ip>/dev/cfg/dns1 | Retrieve or set DNS address 1 |
| http://<miniserver-ip>/dev/cfg/dns2 | Retrieve or set DNS address 2 |
| http://<miniserver-ip>/dev/cfg/ntp | Retrieve or set NTP address |
| http://<miniserver-ip>/dev/cfg/timezoneoffset | Retrieve or set time zone offset |
| http://<miniserver-ip>/dev/cfg/http | Retrieve or set HTTP port |
| http://<miniserver-ip>/dev/cfg/ftp | Retrieve or set FTP port |
| http://<miniserver-ip>/dev/cfg/ftllocalonly | Retrieve or set local access to “FTP, Telnet and local software” |

---

### 3. System Monitoring Commands

These commands provide information about system performance and statistics.

| Command | Function |
| --- | --- |
| http://<miniserver-ip>/dev/sys/numtasks | Retrieve number of tasks |
| http://<miniserver-ip>/dev/sys/cpu | Retrieve CPU load |
| http://<miniserver-ip>/dev/sys/heap | Retrieve memory usage |
| http://<miniserver-ip>/dev/sys/ints | Retrieve number of system interrupts |

---

### 4. Date & Time Management

Manage system date and time settings.

| Command | Function |
| --- | --- |
| http://<miniserver-ip>/dev/sys/date | Retrieve local date |
| http://<miniserver-ip>/dev/sys/time | Retrieve local time |
| http://<miniserver-ip>/dev/sys/setdatetime | Set system date & time (Format: YYYY-MM-DD HH:MM:SS or DD/MM/YYYY HH:MM:SS) |
| http://<miniserver-ip>/dev/sys/ntp | Force NTP request |

---

### 5. Task Monitoring

Retrieve status and properties of system tasks.

| Command | Function |
| --- | --- |
| http://<miniserver-ip>/dev/task0/name | Retrieve Task 0 name |
| http://<miniserver-ip>/dev/task0/priority | Retrieve Task 0 priority |
| http://<miniserver-ip>/dev/task0/stack | Retrieve Task 0 stack usage |
| http://<miniserver-ip>/dev/task0/contextswitches | Retrieve amount of Task 0 context switches |
| http://<miniserver-ip>/dev/task0/waittimeout | Retrieve Task 0 wait time (ms) |
| http://<miniserver-ip>/dev/task0/state | Retrieve Task 0 status |

---

### 6. System Control & File Management

These commands allow rebooting the Miniserver and managing system files.

| Command | Function |
| --- | --- |
| http://<miniserver-ip>/dev/sys/reboot | Reboot Miniserver |
| http://<miniserver-ip>/dev/sys/wsextension/<serial>/Reboot | Reboot a specific extension |
| http://<miniserver-ip>/dev/sys/check | Check active connections |
| http://<miniserver-ip>/dev/sys/logoff | Log out all connections |
| http://<miniserver-ip>/dev/sys/sdtest | Test SD card |
| http://<miniserver-ip>/dev/fslist/ | List root directory of SD card |
| http://<miniserver-ip>/dev/fslist/path/ | List specific directory path |
| http://<miniserver-ip>/dev/fsget/filepath/ | Retrieve a file from SD card |
| http://<miniserver-ip>/dev/fsdel/filepath/ | Delete a file from SD card |

---

### 7. System Logs & Statistics

Retrieve logs and system data.

| Command | Function |
| --- | --- |
| http://<miniserver-ip>/data/status | Output system state in XML |
| http://<miniserver-ip>/stats | Show system statistics |
| http://<miniserver-ip>/dev/fsget/log/def.log | Retrieve log file |
| http://<miniserver-ip>/dev/sys/updatetolatestrelease | Update Miniserver to latest firmware |

---

### 8. External Web Service Execution

Send web service requests to external devices.

| Command | Function |
| --- | --- |
| http://<miniserver-ip>/jdev/sys/sendwebservice?json={"address":"192.168.1.70", "webservice":"/monitor/logs/lwsd"} | Send web service request to another device in network from the Miniserver |

###

## Commands for Devices

### **Extensions**

The following commands apply to **Extensions**.

To execute them, a user with full access rights is required.

| http://<miniserver-ip>/dev/sys/wsextension/<serial>/ForceUpdate/0C000001/DeviceIndex | Forces a firmware update for a specific device. |
| --- | --- |
| http://<miniserver-ip>/dev/sys/wsextension/<serial>/Reboot | Restarts the specified device. |

### **Tree and Air Devices**

The following commands apply to **specific Air or Tree devices**.

To execute them, a user with full access rights is required.

| http://<miniserver-ip>/dev/sys/wsdevice/<serial>/ForceUpdate/0C000001/DeviceIndex | Force firmware update for a specific device |
| --- | --- |
| http://<miniserver-ip>/dev/sys/wsdevice/<serial>/Reboot | Restart the specified device |

					The device serial number (without colons) must be used in place of <serial>.
Alternatively, the device name from Loxone Config can be used.
If using the device name, it must be unique and must not contain special characters.

### Using the Miniserver Web Interface in an iFrame

By default, the Miniserver **does not allow embedding in an iFrame** for security reasons. To enable this functionality, you must explicitly allow it using the following web service command:

http://<miniserver-ip>/dev/sys/allowhttpiframe/1