# Schema Validation Report

## Summary

| Metric | Value |
|--------|-------|
| Total types | 163 |
| Types with connectors | 161 |
| Total connectors (I+O+P) | 1610 |
| UX validated | 2 |
| Source | KB articles + XML config |

## Categories

### Logic (4 types, 11 connectors)

| Type | KB Slug | I | O | P | Total | UX ✓ |
|------|---------|---|---|---|-------|------|
| And |  | 0 | 0 | 0 | 3 |  |
| Or |  | 0 | 0 | 0 | 3 |  |
| Not |  | 0 | 0 | 0 | 2 |  |
| Xor |  | 0 | 0 | 0 | 3 |  |

### Math (10 types, 35 connectors)

| Type | KB Slug | I | O | P | Total | UX ✓ |
|------|---------|---|---|---|-------|------|
| Add2 |  | 0 | 0 | 0 | 3 |  |
| Add4 |  | 0 | 0 | 0 | 5 |  |
| Subtract |  | 0 | 0 | 0 | 3 |  |
| Multiply |  | 0 | 0 | 0 | 3 |  |
| Divide |  | 0 | 0 | 0 | 4 |  |
| Modulo |  | 0 | 0 | 0 | 4 |  |
| Formula |  | 0 | 0 | 0 | 2 |  |
| Average |  | 0 | 0 | 0 | 2 |  |
| MovingAverage |  | 0 | 0 | 0 | 6 |  |
| Integer |  | 0 | 0 | 0 | 3 |  |

### Encoding (5 types, 16 connectors)

| Type | KB Slug | I | O | P | Total | UX ✓ |
|------|---------|---|---|---|-------|------|
| BinaryDecoder |  | 0 | 0 | 0 | 2 |  |
| BinaryDecoder2 |  | 0 | 0 | 0 | 2 |  |
| BinaryEncoder |  | 0 | 0 | 0 | 2 |  |
| AnalogMultiplexer2 |  | 0 | 0 | 0 | 4 |  |
| AnalogMultiplexer4 |  | 0 | 0 | 0 | 6 |  |

### Comparison (8 types, 39 connectors)

| Type | KB Slug | I | O | P | Total | UX ✓ |
|------|---------|---|---|---|-------|------|
| Comparator |  | 0 | 0 | 0 | 5 |  |
| ThresholdSwitch |  | 0 | 0 | 0 | 5 |  |
| DiffThresholdSwitch |  | 0 | 0 | 0 | 5 |  |
| MinMax |  | 0 | 0 | 0 | 4 |  |
| MinMaxReset |  | 0 | 0 | 0 | 5 |  |
| AnalogLimiter |  | 0 | 0 | 0 | 4 |  |
| AnalogWatchdog |  | 0 | 0 | 0 | 3 |  |
| ValueValidator |  | 0 | 0 | 0 | 8 |  |

### Stateful (11 types, 48 connectors)

| Type | KB Slug | I | O | P | Total | UX ✓ |
|------|---------|---|---|---|-------|------|
| Memory | analogue-memory | 1 | 2 | 0 | 3 |  |
| MemoryFlags |  | 0 | 0 | 0 | 1 |  |
| FlipFlopRS |  | 0 | 0 | 0 | 5 |  |
| FlipFlopSR |  | 0 | 0 | 0 | 5 |  |
| Counter |  | 0 | 0 | 0 | 5 |  |
| UpDownCounter |  | 0 | 0 | 0 | 6 |  |
| Stepper |  | 0 | 0 | 0 | 5 |  |
| ShiftRegister |  | 0 | 0 | 0 | 5 |  |
| EdgeDetection |  | 0 | 0 | 0 | 5 |  |
| Status |  | 0 | 0 | 0 | 3 |  |
| StatusMonitor |  | 0 | 0 | 0 | 5 |  |

### Timing (12 types, 46 connectors)

| Type | KB Slug | I | O | P | Total | UX ✓ |
|------|---------|---|---|---|-------|------|
| Monoflop |  | 0 | 0 | 0 | 4 |  |
| PulseGenerator |  | 0 | 0 | 0 | 4 |  |
| PulseAt |  | 0 | 0 | 0 | 4 |  |
| PulseBy |  | 0 | 0 | 0 | 2 |  |
| DelayedPulse |  | 0 | 0 | 0 | 3 |  |
| SwitchOnDelay |  | 0 | 0 | 0 | 4 |  |
| SwitchOnOffDelay |  | 0 | 0 | 0 | 5 |  |
| SavingSwitchOnDelay |  | 0 | 0 | 0 | 4 |  |
| PWM |  | 0 | 0 | 0 | 3 |  |
| WipingRelay |  | 0 | 0 | 0 | 4 |  |
| RandomGenerator |  | 0 | 0 | 0 | 4 |  |
| RandomController |  | 0 | 0 | 0 | 5 |  |

### Other (113 types)

| Type | KB Slug | Connectors | UX ✓ |
|------|---------|------------|------|
| ACCentral | ac-central-controller | 11 |  |
| ACControl | ac-control | 7 |  |
| AalSmartAlarm | aal-smart-alarm | 8 |  |
| AccessController | access-controller | 6 |  |
| Alarm | burglar-alarm | 27 |  |
| AlarmChain | alarm-chain | 10 |  |
| AlarmClock | alarm-clock | 26 | ✅ |
| AudioCentral | audio-central | 26 |  |
| AudioPlayer | audio-player | 37 |  |
| AudioPlayerFixedGroup | audio-player-fixed-group | 18 |  |
| AutoPilot | automatic-rule | 2 |  |
| CallGenerator | call-generator | 7 |  |
| CentralJalousie | automatic-blinds-integrated | 35 |  |
| ClimateController | climate-controller | 29 |  |
| CombinedWindowContact | combined-window-contact | 4 |  |
| ComfortSwitch | multifunction-switch | 8 |  |
| CommandRecognition | command-recognition | 2 |  |
| DaylightControl | brightness-control | 6 |  |
| DewpointCalculator | dewpoint-calculator | 4 |  |
| Dimmer | dimmer | 13 |  |
| DoorController | door-controller | 7 |  |
| DoorWindowMonitor | door-window-monitor | 8 |  |
| EIBDimmer | eib-dimmer | 9 |  |
| EIBJalousie | eib-shading | 6 |  |
| EIBPushbutton | eib-push-button | 8 |  |
| EmergencyAlarm | emergency-alarm | 8 |  |
| EnergyFlowMonitor | energy-flow-monitor | 7 |  |
| EnergyManager | energy-manager | 6 |  |
| EnergyManager2 | energy-manager-2 | 12 |  |
| EnergyMonitor | energy-monitor | 8 |  |
| EventDBConnector | event-database-connector | 21 |  |
| FireWaterAlarm | fire-water-alarm | 15 |  |
| FixedValueMeter | fixed-value-meter | 4 |  |
| FlowTemperature | intelligent-temperature-controller | 14 |  |
| GateCentral | gate-overview | 10 |  |
| GateController | garagegate | 20 |  |
| HVACController | hvac-controller | 16 |  |
| HeatingCurve | heating-curve | 7 |  |
| HotelLightController | hotel-lighting-controller | 29 |  |
| IRController | ir-control | 1 |  |
| IRoomControllerV2 | intelligent-room-controller | 15 |  |
| IntercomV2 | intercom-2 | 11 |  |
| InternormVentilation | internorm-ventilation | 19 |  |
| Irrigation | irrigation | 12 |  |
| Jalousie | automatic-blinds | 41 |  |
| JalousieCentral | shading-overview | 16 |  |
| LeafVentilation | leaf-ventilation | 19 |  |
| LightCentral | lighting-overview | 13 |  |
| LightController2 | lighting-controller | 41 | ✅ |
| LightsceneRGB | rgb-scene-controller | 12 |  |
| LoadManager | load-manager | 14 |  |
| LongClick | long-click | 8 |  |
| MailBox | mail-and-parcel-box | 6 |  |
| MailGenerator | mail-generator | 6 |  |
| MediaController | media-controller | 15 |  |
| Meter | meter | 5 |  |
| MeterBidirectional | meter-bidirectional | 7 |  |
| MeterStorage | meter-storage | 7 |  |
| MixingValve | mixing-valve-controller | 12 |  |
| MultiClick | multiclick | 9 |  |
| MusicServerZone | music-server-zone | 46 |  |
| NFCCodeTouch | authentication-nfc-code-touch | 17 |  |
| PIController | pi-controller | 11 |  |
| PIDController | pid-controller | 11 |  |
| PVForecast | pv-production-forecast | 8 |  |
| Ping | ping-function-block | 5 |  |
| PoolController | pool-controller | 33 |  |
| PositionController2 | 2-position-controller | 6 |  |
| PositionController3 | 3-position-controller | 6 |  |
| PowerSupplyBackup | power-supply-backup-block | 6 |  |
| PresenceDetector | presence | 19 |  |
| PulseMeter | pulse-meter | 6 |  |
| PulseMeterBidirectional | pulse-meter-bidirectional | 9 |  |
| PulseMeterStorage | pulse-meter-storage | 9 |  |
| PushSwitch | push-switch | 7 |  |
| Pushbutton | push-button | 7 |  |
| RadioButtons | radio-buttons | 8 |  |
| RadioButtons16 | radio-buttons-16x | 8 |  |
| RampController | ramp-controller | 6 |  |
| RuntimeCounter | maintenance-counter | 7 |  |
| Sauna | sauna-controller | 25 |  |
| SaunaEvaporator | sauna-controller-evaporator | 35 |  |
| Scaler | scaler | 6 |  |
| Scene | scene | 2 |  |
| Script | custom-script-programming | 4 |  |
| SecurityCentral | security-overview | 11 |  |
| SelectionSwitchOnOff | selection-switch-onoff | 6 |  |
| SelectionSwitchPlus | selection-switch-plus | 8 |  |
| SelectionSwitchPlusMinus | selection-switch-plus-minus | 9 |  |
| Sequencer | sequencer | 8 |  |
| SequentialController | sequential-controller | 4 |  |
| SessionDBConnector | session-database-connector | 23 |  |
| SkyWindow | skylight | 15 |  |
| SkyWindowJalousie | skylight-blinds | 32 |  |
| SolarPumpController | solar-pump-controller | 18 |  |
| SpotPriceOptimizer | spot-price-optimizer | 15 |  |
| StairwellLight | stairwell-light-switch | 7 |  |
| Switch | switch | 5 |  |
| Tablet | tablet | 9 |  |
| TextGenerator | text-generator | 8 |  |
| TimerScheduler | timerscheduler | 10 |  |
| ToiletVentilation | toilet-ventilation-controller | 10 |  |
| TouchGrillAir | touch-and-grill-air | 0 |  |
| TouchGrillControl | touch-and-grill-control | 12 |  |
| TouchNightlight | touch-nightlight-air | 0 |  |
| TouchPureFlex | touch-pure-flex-controller | 22 |  |
| UtilityMeter | utility-meter | 10 |  |
| Ventilation | ventilation | 24 |  |
| Wallbox | wallbox | 10 |  |
| WallboxBlock | wallbox-block | 18 |  |
| WallboxManager | wallbox-manager | 9 |  |
| WindGauge | wind-gauge | 5 |  |
| WindowCentral | window-central | 10 |  |

## Detailed Connector Maps (Top 20 Complex)

### MusicServerZone (`music-server-zone`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| A | A | input | Alarm input |
| AIs | AIs | input | Casatunes Music Server: Source |
| AIv | AIv | input | Volume level |
| API | API | output | API Connector |
| AQr | AQr | output | Remaining time of sleep timer |
| Be | Be | input | Doorbell input |
| Buzzer | Buzzer | input | Alarm clock input |
| Dis | Dis | input | Disable |
| DisMo | DisMo | input | Disable motion sensor |
| FA | FA | input | Fire Alarm |
| MT | MT | parameter | Automatic switch off motion |
| Mo | Mo | input | Motion sensor |
| Mute | Mute | input | Mute |
| Pause | Pause | input | Pause playback |
| Play | Play | input | Start playback |
| R | R | input | Reset |
| ROff | ROff | parameter | Ignore room off command |
| Repeat | Repeat | input | Repeat |
| Rf | Rf | parameter | First recurrence |
| Rr | Rr | parameter | Repeat interval |
| S+ | S+ | input | Casatunes Music Server: Next Source |
| Shuffle | Shuffle | input | Shuffle |
| Sleep | Sleep | input | Sleep timer input |
| Song+ | Song+ | input | Next track |
| Song- | Song- | input | Previous track |
| Stop | Stop | input | Stop playback |
| Sv | Sv | parameter | Volume step size |
| T5 | T5 | input | Combined button input |
| TH | TH | parameter | Duration On |
| TTS | TTS | input | Text to Speech input |
| Tdc | Tdc | parameter | Double click interval |
| Te | Te | parameter | Event playback duration [s] |
| TgZ | TgZ | input | Toggle Zone |
| Ti | Ti | parameter | Delay of the Motion Sensor |
| Tqo | Tqo | parameter | Delay of Qoff [s] |
| Ts | Ts | parameter | Sleep timer duration [s] |
| V+ | V+ | input | Casatunes Music Server: Volume |
| V- | V- | input | Volume |
| Va | Va | parameter | Loxone Music Server: Alarm volume |
| Vbe | Vbe | parameter | Loxone Music Server: Doorbell volume |
| Vbu | Vbu | parameter | Loxone Music Server: Alarm clock volume |
| Vd | Vd | parameter | Startup volume |
| Vm | Vm | parameter | Maximum volume level |
| Vt | Vt | parameter | Loxone Music Server: TTS volume |
| Zoff | Zoff | input | Zone Off |
| Zon | Zon | input | Zone On |

### Jalousie (`automatic-blinds`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| AQpp | AQpp | output | Command output |
| Bld | Bld | parameter | Backlash duration |
| Bldo | Bldo | parameter | Backlash duration opposite |
| Cc | Cc | input | Complete close |
| Cl | Cl | output | Close |
| Co | Co | input | Complete open |
| Dir | Dir | parameter | Compass direction |
| DisPc | DisPc | input | Disable periphery control |
| DisSp | DisSp | input | Disable sun position automatic |
| Dte | Dte | parameter | Direction tolerance end |
| Dts | Dts | parameter | Direction tolerance start |
| Dwc | Dwc | input | Door/window contact |
| Mld | Mld | parameter | Motor lock duration |
| Off | Off | output | Off |
| Op | Op | output | Open |
| Pos | Pos | output | Position of shading |
| Rd | Rd | parameter | Return duration |
| Rdd | Rdd | parameter | Reference Drive Down |
| Sd | Sd | parameter | Slat distance |
| Slat | Slat | output | Position of slats |
| So | So | input | Slightly open |
| Sp | Sp | output | Sun position automatic |
| Spe | Spe | parameter | Sun position automatic end action |
| Spi | Spi | parameter | Sun position automatic interval |
| Spm | Spm | parameter | Sun position automatic mode |
| Spoe | Spoe | parameter | Sun position automatic end offset |
| Spos | Spos | parameter | Sun position automatic start offset |
| Spr | Spr | input | Sun position automatic restart |
| Sps | Sps | input | Sun position automatic start |
| Sw | Sw | parameter | Slat width |
| T5 | T5 | input | T5 control |
| TPos | TPos | output | Target position |
| Tdc | Tdc | parameter | Time double-click |
| Tg | Tg | input | Toggle |
| Tlc | Tlc | parameter | Time long-click |
| Type | Type | parameter | Shading type |
| Wa | Wa | input | Wind alarm |
| Wap | Wap | parameter | Wind alarm position |
| Wds | Wds | output | Wind, door/window contact state |
| minTd | minTd | parameter | Minimum travel duration |

### LightController2 (`lighting-controller`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| AQ1 | AQ1 | output | Analog output 1 |
| Alarm | Alarm | output | Alarm |
| AlarmClock | Buzzer | output | Buzzer |
| AlarmClockPeriod | Fbu | parameter | Fading time buzzer |
| AlarmPeriod | Afi | parameter | Alarm flashing interval |
| Brightness | Br | output | Current brightness |
| BrightnessLimit | Brt | parameter | Brightness threshold |
| EnMove | Mo | input | Motion |
| FadingTime | Ft | parameter | Fading time |
| I1 | Lc1 | input | Light circuit 1 |
| I2 | Lc2 | input | Light circuit 2 |
| I3 | Lc3 | input | Light circuit 3 |
| I4 | Lc4 | input | Light circuit 4 |
| InputDisable | DisP | input | Disable presence/motion |
| InputTriggerDown | DisPc | output | Disable periphery control |
| MasterBr | MBr | output | Master Brightness |
| Max | MaxBr | parameter | Maximum brightness |
| Min | MinBr | parameter | Minimum brightness |
| Move | M- | input | Previous mood |
| MoveOn | On | output | All on |
| MoveTimeout | Moet | parameter | Motion extend time |
| On | Off | input | Off |
| OutputAPI | API | output | API Connector |
| OutputReset | 2C | output | Pulse on double-click |
| OutputResetAll | 3C | output | Pulse on triple-click |
| Presence | P | output | Presence |
| Remanence | Rem | parameter | Remanence input |
| RtD | Rtd | output | Reset to default |
| Scene | M | output | Current mood |
| SceneMixTime | Mmd | parameter | Mixing moods duration |
| Sel1 | T5/1 | input | Trigger Mood 1 |
| Sel2 | T5/2 | input | Trigger Mood 2 |
| Sel3 | T5/3 | input | Trigger Mood 3 |
| Sel4 | T5/4 | input | Trigger Mood 4 |
| Sel5 | T5/5 | input | Trigger Mood 5 |
| Sel6 | T5/6 | input | Trigger Mood 6 |
| Sel7 | T5/7 | input | Trigger Mood 7 |
| Sel8 | T5/8 | input | Trigger Mood 8 |
| Select | Mood | input | Select mood by ID |
| Step | M+ | input | Next mood |
| Steptime | Str | parameter | Step rate brightness |

### AudioPlayer (`audio-player`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| 2C | 2C | output | Pulse on double-click |
| 3C | 3C | output | Pulse on triple-click |
| API | API | output | API Connector |
| Alarm | Alarm | input | Alarm |
| BTp | BTp | parameter | Bluetooth pairing |
| Bell | Bell | input | Bell |
| Buzzer | Buzzer | input | Buzzer |
| BuzzerFav | BuzzerFav | parameter | Alarm clock favorite |
| Cs | Cs | input | Custom sound |
| DisP | DisP | input | Disable presence |
| DisPc | DisPc | input | Disable periphery control |
| Fav | Fav | input | Set favorite |
| FireAlarm | FireAlarm | input | Fire alarm |
| Ft | Ft | parameter | Volume Fading Time |
| LineIn | LineIn | input | Set Line In |
| Off | Off | input | Off |
| P | P | input | Presence |
| Play | Play | output | Play status |
| Roff | Roff | parameter | Ignore room off command |
| Rtd | Rtd | input | Reset to default |
| Stereo L | Stereo L | output | Stereo Left |
| Stereo LR | Stereo LR | output | Stereo Left & Right |
| Stereo R | Stereo R | output | Stereo Right |
| Sub | Sub | output | Subwoofer |
| T5 | T5 | input | T5 control |
| TTS | TTS | input | Text to speech |
| Tg | Tg | input | Toggle |
| V | V | input | Set volume |
| V+ | V+ | output | Pulse on Volume+ |
| V- | V- | output | Pulse on Volume- |
| Va | Va | parameter | Minimum volume alarm sounds |
| Vbell | Vbell | parameter | Minimum volume doorbell |
| Vbuzzer | Vbuzzer | parameter | Minimum volume alarm clock |
| Vm | Vm | parameter | Maximum volume |
| Von | Von | parameter | Power on volume |
| Vsts | Vsts | parameter | Step size volume |
| Vtts | Vtts | parameter | Minimum volume TTS and announcements |

### CentralJalousie (`automatic-blinds-integrated`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| Blk | Blk | output | Motor blocked |
| Cc | Cc | input | Complete close |
| Co | Co | input | Complete open |
| Dir | Dir | parameter | Compass direction |
| DisPc | DisPc | input | Disable periphery control |
| DisSp | DisSp | input | Disable sun position automatic |
| Dte | Dte | parameter | Direction tolerance end |
| Dts | Dts | parameter | Direction tolerance start |
| Dwc | Dwc | input | Door/window contact |
| Im | Im | output | In motion |
| Obs | Obs | output | Obstacle |
| Off | Off | output | Off |
| Pos | Pos | output | Position of shading |
| Sd | Sd | parameter | Slat distance |
| Slat | Slat | output | Position of slats |
| So | So | input | Slightly open |
| Sop | Sop | parameter | Slightly open position |
| Sp | Sp | output | Sun position automatic |
| Spe | Spe | parameter | Sun position automatic end action |
| Spi | Spi | parameter | Sun position automatic interval |
| Spm | Spm | parameter | Sun position automatic mode |
| Spoe | Spoe | parameter | Sun position automatic end offset |
| Spos | Spos | parameter | Sun position automatic start offset |
| Spr | Spr | input | Sun position automatic restart |
| Sps | Sps | input | Sun position automatic start |
| Spu | Spu | parameter | Slat position upwards movement |
| Sw | Sw | parameter | Slat width |
| T5 | T5 | input | T5 control |
| Tdc | Tdc | parameter | Time double-click |
| Tg | Tg | input | Toggle |
| Tlc | Tlc | parameter | Time long-click |
| Wa | Wa | input | Wind alarm |
| Wap | Wap | parameter | Wind alarm position |
| Wds | Wds | output | Wind, door/window contact state |

### SaunaEvaporator (`sauna-controller-evaporator`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| Dc | Dc | input | Door contact |
| DisPc | DisPc | input | Disable periphery control |
| Dryd | Dryd | parameter | Drying phase duration |
| Dryϑ | Dryϑ | parameter | Drying phase temperature |
| Ev | Ev | output | Evaporator output (0-10V) |
| Evd | Evd | output | Evaporator digital output |
| Fan | Fan | output | Fan |
| Fim | Fim | input | Finnish manual |
| Fin | Fin | input | Finnish sauna |
| G | G | parameter | Gain |
| Her | Her | input | Herbal sauna |
| Hot | Hot | input | Hot-air bath |
| Ht | Ht | output | Target humidity |
| Hum | Hum | input | Humidity manual |
| L1-3 | L1-3 | output | Sauna phase output (1-3) |
| Mode | Mode | output | Current sauna mode |
| Off | Off | input | Off |
| On | On | output | Sauna state |
| P | P | input | Presence |
| PWMp | PWMp | parameter | PWM period |
| Pm | Pm | parameter | Phase mode |
| Ready | Ready | output | Sauna ready |
| So | So | output | Sauna output (0-10V) |
| Sof | Sof | input | Soft Steam bath |
| Ssd | Ssd | output | Safety shutdown |
| Ssdt | Ssdt | parameter | Safety shutdown time |
| Ssdϑ | Ssdϑ | parameter | Safety shutdown temperature |
| St | St | output | Sand timer state |
| Stoff | Stoff | output | Sand timer end |
| Tg | Tg | input | Toggle |
| Ws | Ws | input | Water shortage |
| ϑb | ϑb | input | Current temperature bench |
| ϑd | ϑd | parameter | Temperature deviation |
| ϑt | ϑt | output | Target temperature |

### PoolController (`pool-controller`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| Bw | Bw | output | Backwash cycle state |
| Bwd | Bwd | parameter | Backwash cycle duration |
| C | C | output | Cooling demand |
| Ci | Ci | output | Circulation cycle state |
| Cid | Cid | parameter | Circulation cycle duration |
| Cl | Cl | output | Close pool cover |
| Cpos | Cpos | input | Cover position |
| Cϑm | Cϑm | output | Cycle started via (ϑm) |
| DisPc | DisPc | input | Disable periphery control |
| Dr | Dr | output | Draining cycle state |
| Drd | Drd | parameter | Draining cycle duration |
| Dv | Dv | output | Drain valve |
| Eco | Eco | input | Eco |
| Error | Error | output | Error code |
| Fi | Fi | output | Filtration cycle state |
| Fid | Fid | parameter | Filtration cycle duration |
| Fp | Fp | output | Filtration pump |
| Fpet | Fpet | parameter | Filtration pump extend time |
| H | H | output | Heating demand |
| I1 | I1 | input | Custom input 1 |
| I2 | I2 | input | Custom input 2 |
| Off | Off | input | Off |
| Om | Om | output | Current operating mode |
| Op | Op | output | Open pool cover |
| Rid | Rid | parameter | Rinsing cycle duration |
| Rtc | Rtc | output | Remaining duration of the active cycle |
| Sm | Sm | input | Swimming machine |
| Vp | Vp | input | Set valve position |
| Vpos | Vpos | output | Current valve position |
| Wm | Wm | output | Water machine |
| ϑh | ϑh | parameter | Hysteresis temperature |
| ϑm | ϑm | output | Current temperature control mode |

### SkyWindowJalousie (`skylight-blinds`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| Cc | Cc | input | Complete close |
| Cl | Cl | output | Close |
| Co | Co | input | Complete open |
| Dir | Dir | parameter | Compass direction |
| DisPc | DisPc | input | Disable periphery control |
| DisSp | DisSp | input | Disable sun position automatic |
| Dte | Dte | parameter | Direction tolerance end |
| Dts | Dts | parameter | Direction tolerance start |
| Dwc | Dwc | input | Door/window contact |
| Mld | Mld | parameter | Motor lock duration |
| Off | Off | output | Off |
| Op | Op | output | Open |
| Pi | Pi | parameter | Pitch |
| Pos | Pos | output | Position of shading |
| So | So | input | Slightly open |
| Sop | Sop | parameter | Slightly open position |
| Sp | Sp | output | Sun position automatic |
| Spe | Spe | parameter | Sun position automatic end action |
| Spm | Spm | parameter | Sun position automatic mode |
| Spoe | Spoe | parameter | Sun position automatic end offset |
| Spos | Spos | parameter | Sun position automatic start offset |
| Spr | Spr | input | Sun position automatic restart |
| Sps | Sps | input | Sun position automatic start |
| T5 | T5 | input | T5 control |
| Tdc | Tdc | parameter | Time double-click |
| Tg | Tg | input | Toggle |
| Tlc | Tlc | parameter | Time long-click |
| Wa | Wa | input | Wind alarm |
| Wap | Wap | parameter | Wind alarm position |
| Wds | Wds | output | Wind, door/window contact state |
| minTd | minTd | parameter | Minimum travel duration |

### ClimateController (`climate-controller`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| Ah | Ah | input | Additional heating |
| B | B | input | Boost |
| C2 | C2 | output | Cooling stage 2 |
| Dfc | Dfc | parameter | Days until Filter Change |
| Doff | Doff | parameter | Duration for Off |
| Don | Don | parameter | Duration for On |
| Ec | Ec | input | Excess cooling |
| Eh | Eh | input | Excess heating |
| F | F | output | Fan |
| Fc | Fc | output | Filter change |
| Fod | Fod | parameter | Fan Overrun Duration |
| H2 | H2 | output | Heating stage 2 |
| MaxTp | MaxTp | parameter | Maximum threshold for pulsing |
| Mh | Mh | input | Manual heating |
| MinHr | MinHr | parameter | Time minimum HVAC Runtime |
| Mode | Mode | parameter | Mode |
| Off | Off | input | Off |
| Otm | Otm | parameter | Outdoor Temperature Mode |
| Sot | Sot | parameter | Switch on threshold |
| Sv | Sv | output | Switching valve |
| Tt2s | Tt2s | parameter | Time to second stage |
| Vd | Vd | parameter | Valve delay |
| ϑLimC | ϑLimC | parameter | Temperature Limit Cooling |
| ϑLimH | ϑLimH | parameter | Temperature Limit Heating |
| ϑminHP | ϑminHP | parameter | Minimum Temperature Heat Pump |
| ϑminS2 | ϑminS2 | parameter | Minimum Temperature Stage 2 |
| ϑo | ϑo | input | Outdoor Temperature |
| ϑoa | ϑoa | output | Average outdoor temperature |

### HotelLightController (`hotel-lighting-controller`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| AIr | AIr | input | Room state |
| AQ1-20 | AQ1-20 | output | Analog output of actuator/dimmer 1-20 on RGB - %-v |
| AQrm | AQrm | output | Analogue output for room service status 1 = Vacant |
| AQs | AQs | output | Analogue Output - Value indicating currently activ |
| Dis | Dis | input | Disable |
| DisMo | DisMo | input | Disable motion sensor input |
| ID | ID | input | Door contact |
| IS | IS | input | Service button |
| L | L | parameter | Do not set last value |
| M | M | parameter | Max time between pulses |
| MS | MS | parameter | Service staff mode |
| Max | Max | parameter | Maximum value |
| Min | Min | parameter | Minimum value |
| Mo | Mo | input | Motion sensor |
| QD | QD | output | Service completed output |
| QP | QP | output | Presence Output |
| QS | QS | output | Service Output |
| R | R | input | Reset |
| Rem | Rem | parameter | Remanence input |
| S10 | S10 | input | Trigger selection scene 10 |
| S11 | S11 | input | Trigger seletion scene 11 |
| S12 | S12 | input | Trigger seletion scene 12 |
| S13 | S13 | input | Trigger seletion scene 13 |
| SI | SI | parameter | Step |
| ST | ST | parameter | Step rate |
| TH | TH | parameter | Duration On |
| TM | TM | parameter | Duration of Service Staff mode |
| Tl | Tl | parameter | Timeout for leaving room |
| To | To | parameter | Duration press and hold for 'All off' |

### Alarm (`burglar-alarm`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| A | A | input | Arm with presence detection |
| API | API | output | API Connector |
| Ad | Ad | input | Arm delayed with presence detection |
| Adnp | Adnp | input | Arm delayed without presence detection |
| Anp | Anp | input | Arm without presence detection |
| Aoc | Aoc | parameter | Arm open contact |
| At | At | output | Alarm test |
| Atm | Atm | parameter | Alarm test mode |
| Ca | Ca | output | Cause of alarm |
| Dc | Dc | input | Door contacts |
| DisPc | DisPc | input | Disable periphery control |
| Eip | Eip | parameter | Extension of alarm input pulses |
| Gb | Gb | input | Glass breakage |
| MaxA | MaxA | parameter | Maximum alarm duration |
| Off | Off | input | Off |
| Ot | Ot | input | Other |
| P | P | input | Presence |
| Rem | Rem | parameter | Remanence input |
| S | S | output | Status |
| Sac | Sac | parameter | Silent alarm confirmation |
| Spt | Spt | parameter | Second presence sensor time window |
| Ta | Ta | output | Time and date of alarm |
| Tg | Tg | input | Toggle with presence detection |
| Tgnp | Tgnp | input | Toggle without presence detection |
| WDot | WDot | output | Text output for open windows / doors |
| WDs | WDs | output | Window / door state |
| Wc | Wc | input | Window contacts |

### AlarmClock (`alarm-clock`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| AQs | AQs | output | Alarm sound output |
| Acknowledge | Ca | input | Confirm alarm |
| AlarmTime | MaxA | parameter | Max alarm duration |
| BrightAct | Ba | parameter | Brightness active |
| BrightInact | Bi | parameter | Brightness inactive |
| Deactivate | DisA | input | Disable alarm entries |
| OutputAPI | API | output | API Connector |
| ParWakeAlarmSloping | Asf | parameter | Alarm fade-in |
| ParWakeAlarmSound | As | parameter | Alarm sound |
| ParWakeAlarmVolume | Asv | parameter | Alarm volume |
| PrepTime | Pat | parameter | Pre-alarm time |
| QMe | QMe | output | Current mode |
| QTa | A | output | Alarm active |
| QTe | Da | output | Default alarm state |
| QTna | QTna | output | Next alarm time |
| QTp | Tna | output | Time until next alarm |
| Qa | Buzzer | output | Buzzer |
| Qat | Pa | output | Pre-alarm |
| Remanence | Aus | remanence | Remanence input |
| Reset | Off | input | Off |
| RtD | Rtd | input | Reset to default |
| Snooze | S | input | Start snooze timer |
| SnoozeTime | Sd | parameter | Snooze duration |
| TMe | Set | input | Set alarm time |
| TgMe | Tg | input | Toggle |
| VIN1 | VIN1 | input | Virtual input (internal) |

### AudioCentral (`audio-central`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| AIs | AIs | input | Casatunes Music Server: Source |
| API | API | output | API Connector |
| Alarm | Alarm | input | Alarm |
| Bell | Bell | input | Bell |
| Buzzer | Buzzer | input | Buzzer |
| Cs | Cs | input | Custom sound |
| DisP | DisP | input | Disable presence |
| DisPc | DisPc | input | Disable periphery control |
| FireAlarm | FireAlarm | input | Fire alarm |
| Na | Na | output | Active Audio Players |
| Off | Off | input | Off |
| Repeat | Repeat | input | Repeat |
| Roff | Roff | parameter | Ignore room off command |
| Rtd | Rtd | input | Reset to default |
| S+ | S+ | input | Casatunes Music Server: Next Source |
| Shuffle | Shuffle | input | Shuffle |
| Sleep | Sleep | input | Sleep timer input |
| Stop | Stop | input | Stop playback |
| T5 | T5 | input | T5 control |
| TTS | TTS | input | Text to Speech input |
| TgZ | TgZ | input | Toggle Zone |
| V | V | input | Set volume |
| V+ | V+ | input | Volume+ |
| V- | V- | input | Volume- |
| Zoff | Zoff | input | Zone Off |
| Zon | Zon | input | Zone On |

### Sauna (`sauna-controller`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| Dc | Dc | input | Door contact |
| DisPc | DisPc | input | Disable periphery control |
| Dry | Dry | output | Drying state |
| Dryd | Dryd | parameter | Drying phase duration |
| Dryϑ | Dryϑ | parameter | Drying phase target temperature |
| Fan | Fan | output | Fan |
| G | G | parameter | Gain |
| L1-3 | L1-3 | output | Sauna phase output (1-3) |
| Off | Off | input | Off |
| On | On | output | Sauna state |
| P | P | input | Presence |
| PWMp | PWMp | parameter | PWM period |
| Pm | Pm | parameter | Phase mode |
| Ready | Ready | output | Sauna ready |
| So | So | output | Sauna output (0-10V) |
| Ssd | Ssd | output | Safety shutdown |
| Ssdt | Ssdt | parameter | Safety shutdown time |
| Ssdϑ | Ssdϑ | parameter | Safety shutdown temperature |
| St | St | output | Sand timer state |
| Stoff | Stoff | output | Sand timer end |
| Tg | Tg | input | Toggle |
| ϑb | ϑb | input | Current temperature bench |
| ϑd | ϑd | parameter | Temperature deviation |
| ϑt | ϑt | output | Target temperature |

### Ventilation (`ventilation`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| B | B | input | Boost |
| Bva | Bva | parameter | Basic ventilation absence |
| Bvp | Bvp | parameter | Basic ventilation presence |
| CO2 | CO2 | input | CO2 indoor. |
| CO2max | CO2max | parameter | Maximum CO2 (air pollution) |
| Dwc | Dwc | input | Door/window contact |
| Ex | Ex | input | Exhaust air |
| F | F | output | Fan |
| Fea | Fea | output | Fan exhaust air |
| Fpt | Fpt | parameter | Frost protection temperature |
| Fsa | Fsa | output | Fan supply air |
| He | He | output | Heat exchanger |
| Hi | Hi | input | Humidity indoor. |
| Hmax | Hmax | parameter | Maximum humidity |
| Iva | Iva | parameter | Intensive ventilation absence |
| Ivp | Ivp | parameter | Intensive ventilation presence |
| Off | Off | input | Off |
| Pet | Pet | parameter | Presence extend time |
| Rtd | Rtd | input | Reset to default |
| S | S | output | Status |
| Sat | Sat | input | Supply air temperature |
| Sm | Sm | input | Sleep mode |
| Smt | Smt | parameter | Sleep mode timeout |

### SessionDBConnector (`session-database-connector`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| CI1 | CI1 | input | Custom input 1 |
| CI10 | CI10 | input | Custom input 10 |
| CI11 | CI11 | input | Custom input 11 |
| CI12 | CI12 | input | Custom input 12 |
| CI13 | CI13 | input | Custom input 13 |
| CI14 | CI14 | input | Custom input 14 |
| CI15 | CI15 | input | Custom input 15 |
| CI16 | CI16 | input | Custom input 16 |
| CI2 | CI2 | input | Custom input 2 |
| CI3 | CI3 | input | Custom input 3 |
| CI4 | CI4 | input | Custom input 4 |
| CI5 | CI5 | input | Custom input 5 |
| CI6 | CI6 | input | Custom input 6 |
| CI7 | CI7 | input | Custom input 7 |
| CI8 | CI8 | input | Custom input 8 |
| CI9 | CI9 | input | Custom input 9 |
| Log | Log | output | Log output |
| SEnd | SEnd | input | Session end |
| SStart | SStart | input | Session start |
| Sa | Sa | output | Session active |
| Td | Td | parameter | Trigger Delay |
| Uid | Uid | input | User-ID |

### TouchPureFlex (`touch-pure-flex-controller`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| B | B | output | Bright |
| Bl | Bl | input | Backlight On |
| Br | Br | output | Room brightness |
| BrBP | BrBP | parameter | Bright and Presence Brightness |
| BrBnP | BrBnP | parameter | Bright and no Presence Brightness |
| BrD | BrD | output | Brightness Display & status LEDs |
| BrDP | BrDP | parameter | Dark and Presence Brightness |
| BrDef | BrDef | parameter | Default Brightness |
| BrDnP | BrDnP | parameter | Dark and no Presence Brightness |
| BrLP | BrLP | parameter | Lighting and Presence Brightness |
| BrLnP | BrLnP | parameter | Lighting and no Presence Brightness |
| Brt | Brt | parameter | Brightness threshold |
| CBrB | CBrB | output | Current Brightness Backlight |
| DnD | DnD | input | Do not Disturb |
| Don | Don | input | Display On |
| L | L | output | Lighting Active |
| LbT | LbT | input | Light by Touch |
| Off | Off | input | Off |
| P | P | output | Presence |
| Set | Set | input | Set Brightness |
| Txt | Txt | input | Display Text |

### EventDBConnector (`event-database-connector`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| CI1 | CI1 | input | Custom input 1 |
| CI10 | CI10 | input | Custom input 10 |
| CI11 | CI11 | input | Custom input 11 |
| CI12 | CI12 | input | Custom input 12 |
| CI13 | CI13 | input | Custom input 13 |
| CI14 | CI14 | input | Custom input 14 |
| CI15 | CI15 | input | Custom input 15 |
| CI16 | CI16 | input | Custom input 16 |
| CI2 | CI2 | input | Custom input 2 |
| CI3 | CI3 | input | Custom input 3 |
| CI4 | CI4 | input | Custom input 4 |
| CI5 | CI5 | input | Custom input 5 |
| CI6 | CI6 | input | Custom input 6 |
| CI7 | CI7 | input | Custom input 7 |
| CI8 | CI8 | input | Custom input 8 |
| CI9 | CI9 | input | Custom input 9 |
| ETr | ETr | input | Trigger |
| Log | Log | output | Log output |
| Td | Td | parameter | Trigger Delay |
| Uid | Uid | input | User-ID |

### GateController (`garagegate`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| Cc | Cc | input | Complete close |
| Co | Co | input | Complete open |
| DisPc | DisPc | input | Disable periphery control |
| Ic | Ic | input | Is closed |
| Io | Io | input | Is open |
| Mld | Mld | parameter | Motor lock duration |
| Off | Off | input | Off |
| Pd | Pd | parameter | Pulse duration |
| Po | Po | input | Partially Open |
| PoPos | PoPos | parameter | Partially Open Position |
| Pos | Pos | output | Position |
| Ppd | Ppd | parameter | Pulse pause duration |
| Rem | Rem | parameter | Remanence input |
| Spc | Spc | input | Sensor prevent closing |
| Spo | Spo | input | Sensor prevent opening |
| T5 | T5 | input | T5 control |
| Tg | Tg | output | Pulse to Open/Stop/Close |
| Type | Type | parameter | Type |
| Wl | Wl | output | Warning light |

### InternormVentilation (`internorm-ventilation`)

| Key | UX | Dir | Description |
|-----|-----|-----|-------------|
| API | API | output | API Connector |
| B | B | input | Boost |
| Bva | Bva | parameter | Basic ventilation absence |
| Bvp | Bvp | parameter | Basic ventilation presence |
| CO2max | CO2max | parameter | Maximum CO2 (air pollution) |
| Dwc | Dwc | input | Door/window contact |
| Error | Error | output | Error |
| Ex | Ex | input | Exhaust air |
| Fc | Fc | output | Filter change |
| Hmax | Hmax | parameter | Maximum humidity |
| Iva | Iva | parameter | Intensive ventilation absence |
| Ivp | Ivp | parameter | Intensive ventilation presence |
| Off | Off | input | Off |
| Pet | Pet | parameter | Presence extend time |
| Rtd | Rtd | input | Reset to default |
| S | S | output | Status |
| Sat | Sat | input | Supply air temperature |
| Sm | Sm | input | Sleep mode |
| Smt | Smt | parameter | Sleep mode timeout |
