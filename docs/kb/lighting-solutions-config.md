# Lighting Solutions Config

Source: https://www.loxone.com/enen/kb/lighting-solutions-config/

---

## OVERVIEW

With the Loxone system we know that there can be many different ways to create lighting in your home. Here we can now take the devices we have in the system.

[Single Switched Output](#Single Switched Output)

[Single Dimmed Output](#Single Dimmed Output)

[Multiple Dimmed and Switched Outputs (Including 24V](#Multiple Outputs (Switched & Dimmed))[ RGBW Downlights & LED Strip Tape)

](#Multiple Outputs (Switched & Dimmed))[Scenes](#Scenes)

#### Single Switched Output

*[Icon Exclamation Mark Loxone]*Please remember to save as you go, both ‘Save in Miniserver’ and a local back up on your computer.

To setup the Lighting Controller Function block for a single switched circuit you first need to bring open Loxone Config, find a page and press F5. Once a little box pops up, type ‘Lighting’ and you will see the Lighting Controller appear.

*[F5 Searching For Lighting Controller ]*

Double click on the name and the block will be dropped into the page.

*[en_kb_config_lighiting_controller]*

Drag out your Touch Switch (either Touch Air or Touch Tree) and you will see your Lighting Controller will begin to flash. If you drop the Touch Switch directly onto the block, the [T5](https://www.loxone.com/enen/smart-home/switch-standard/) input will automatically appear and be assigned.

*[en_kb_config_flashing_lighiting_controller]*

*[en_kb_config_t5_lighiting_controller]*

Select your respective output and drag this onto the Lighting Controller again. Just like with the input, the Lighting Controller will flash.

*[en_kb_config_switched_output_lighiting_controller]*

Please note that yours maybe different depending on what device you are using (Air/Tree/Miniserver/Extension) but they will still be a digital output.

By default, the [T5](https://www.loxone.com/enen/smart-home/switch-standard/) input will automatically scroll through scenes. As we are not using scenes at the moment, if you double click on the lighting controller and tick ‘All OFF scene (S0)’ this will make sure you switch will toggle the lights on and off.  If you want to learn about scenes you would need a Dimmed output or even better, a mixture of both; follow [here to find out more about scenes](#Scenes).

*[en_kb_config_all_off_disabled]*

Save this into your Miniserver and then check out how the user interface will look.

*[en_kb_ui_switched_output_lighiting_controller]*

#### Single Dimmed Output

*[Icon Exclamation Mark Loxone]*Please remember to save as you go, both ‘Save in Miniserver’ and a local back up on your computer.

To setup the Lighting Controller Function block for a single Dimmed circuit you first need to bring open Loxone Config, find a page and press F5. Once a little box pops up, type ‘Lighting’ and you will see the Lighting Controller appear.

*[F5 Searching For Lighting Controller ]*

Double click on the name and the block will be dropped into the page.

*[en_kb_config_lighiting_controller]*

Drag out your Touch Switch (either Touch Air or Touch Tree) and you will see your Lighting Controller will begin to flash. If you drop the Touch Switch directly onto the block, the [T5](https://www.loxone.com/enen/smart-home/switch-standard/) input will automatically appear and be assigned.

*[en_kb_config_flashing_lighiting_controller]*

*[en_kb_config_t5_lighiting_controller]*

Select your respective dimmed output and drag this onto the Lighting Controller again. Just like with the input, the Lighting Controller will flash.

*[en_kb_config_dimmed_output_lighiting_controller]*

Please note that yours maybe different depending on what device you are using ([Nano Dimmer Air](https://www.loxone.com/enen/kb/nano-dimmer-air/)/[Dimmer Extension](https://www.loxone.com/enen/kb/dimmer-setup/)) but they will still be a digital output.

By default, the [T5](https://www.loxone.com/enen/smart-home/switch-standard/) input will automatically scroll through scenes. As we are not using scenes at the moment, if you double click on the lighting controller and tick ‘All OFF scene (S0)’ this will make sure you switch will toggle the lights on and off.  If you want to learn about scenes you would need a Dimmed output or even better, a mixture of both; follow [here to find out more about scenes](#Scenes).

*[en_kb_config_all_off_disabled]*

Save this into your Miniserver and then check out how the user interface will look.

*[en_kb_ui_dimmed_output_lighiting_controller]*



#### Multiple Outputs (Switched & Dimmed)

*[Icon Exclamation Mark Loxone]*Please remember to save as you go, both ‘Save in Miniserver’ and a local back up on your computer.

To setup the Lighting Controller Function block for a Multiple circuits, both Dimmed and Switched you first need to bring open Loxone Config, find a page and press F5. Once a little box pops up, type ‘Lighting’ and you will see the Lighting Controller appear.

*[F5 Searching For Lighting Controller ]*

Double click on the name and the block will be dropped into the page.

*[en_kb_config_lighiting_controller]*

Drag out your Touch Switch (either Touch Air or Touch Tree) and you will see your Lighting Controller will begin to flash. If you drop the Touch Switch directly onto the block, the [T5](https://www.loxone.com/enen/smart-home/switch-standard/) input will automatically appear and be assigned.

*[en_kb_config_flashing_lighiting_controller]**[en_kb_config_t5_lighiting_controller]*

Select your respective dimmed output and drag this onto the Lighting Controller again. Just like with the input, the Lighting Controller will flash.

*[en_kb_config_dimmed_output_lighiting_controller_side_lamp]*

Repeat this step for your switched outputs and RGB & W (red, green, blue and white).

*[en_kb_config_multiple_output_lighiting_controller]*

You may want to change the output name of a device before saving into the Miniserver. This will keep all the outputs looking neat and presentable.

To do this double click on your Lighting Controller and click on the ‘Outputs’ tab.

*[en_kb_config_changing_output_name_lighiting_controller]*

Change the output names and assign the outputs names that make more sense for device location (remember this is what you or the customer will see on their Loxone application).

*[en_kb_config_changing_output_name_lighiting_controller_2]*

Click ‘Ok’, save this into your Miniserver and then check out how the user interface will look.

*[en_kb_ui_multiple_output_lighiting_controller]*

Now you can create some incredible [scenes](#Scenes) with your multiple outputs!



#### Scenes

*[Icon Exclamation Mark Loxone]*Please remember to save as you go, both ‘Save in Miniserver’ and a local back up on your computer.

Once lighting outputs have been created, various scenes can be made to set the mood for any occasion or setting. If you need assistance on how to add Lighting outputs to your configuration, we suggest reading the above steps.

Scenes are more easily configured on the interface (via phone or web page). By doing this you can also see live how the room looks and feels to the scenes you wish to create.

As an example, let’s create an evening scene with our multiple outputs. On our interface we currently see this:

*[en_kb_ui_multiple_output_lighiting_controller]*

If we change any of the lights, a button appears at the bottom; ‘Save Scene’.

Set the lights to the levels you wish and tap/click ‘Save Scene’ (our example is for a evening lighting scene in the living room, yours may well look different to our room).

*[en_kb_ui_creating_scenes]*

A popup box should appear allowing you to either overwrite a scene already created (by tapping/clicking on the existing scene) or create a new scene by tapping/clicking on a unused space.

*[en_kb_ui_creating_scenes_naming]**[en_kb_ui_creating_scenes_naming_2]*

Once an appropriate name has been chosen, tap/click on the top right tick and as shown below; a new ‘Evening’ scene has been created.

*[en_kb_ui_creating_scenes_2]*

From this you can scroll through the lighting scenes using the [T5](https://www.loxone.com/enen/smart-home/switch-standard/) input or selecting the scene from your phone, tablet or web interface.

By default the scene scroll is as follows:
- Scene 1-8
- Scene 9 (All ON)
- Scene 0 (All OFF)