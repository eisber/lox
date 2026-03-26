# Multiplicator Project

Source: https://www.loxone.com/enen/kb/multiplicator-project/

---

With a Multiplicator Project an identical programming can be distributed to several Miniservers.


    This is suitable for installations with many similar units such as hotel rooms or apartments, up to entire groups of houses.
Each unit gets its own Miniserver including user interface.


    Programming can be adapted at will on the individual Miniservers, as different settings, parameters, devices or programming are often required.


    The entire Multiplicator Project including the Miniservers is managed together in Loxone Config.


    The Multiplicator cannot be used for Gateway/Client configurations.



## Table of Contents
- [Setup a Multiplicator Project](#Setup)
- [Basic Programming](#Basic)
- [Program Miniserver independently](#Programming)
- [Trusts](#Trusts)




---


## Setup a Multiplicator Project



|  | Current Miniserver / Go / Compact required, not supported by Gen. 1 variants!Only Miniservers of the same type can be used in a Multiplicator Project. |
| --- | --- |

    To create a project, select the "Convert to Multiplicator Project" button, and confirm:





The desired Miniservers can now be added via "Miniserver Search":




If the Miniservers are not connected in a local network, they must first be made accessible externally, e.g. via [Remote Connect](https://www.loxone.com/help/remote-connect).
Then they are added manually via the button in the menu bar, and then the connection data with external address is entered.



The current credentials of each Miniserver must be entered on the right side.
These are later overwritten by the Multiplicator Project:



    In the "Manage Miniservers" dialog, the current project is saved on all Miniservers and, if necessary, the firmware is updated.




    After saving, if all Miniservers are reachable and have been updated to the latest firmware, the project is ready for use.



---


## Basic Programming


    A basic program is created, which is later distributed to all Miniservers.


    For example, for the bedroom the Lighting Controller, Intelligent Room Controller and Automatic Shading are added to the programming page of the Multiplicator Project:



    Usually, the same devices are used on each Miniserver in a project of this type. Therefore, placeholders are inserted via "Add Device" for the time being. Later, these devices are paired at the respective Miniservers with their serial number and the placeholders are replaced.


    Individual pages or devices can also be transferred to certain Miniservers:




---


## Program Miniserver independently


    Since the programming on each server differs in detail, it is possible to access each Miniserver separately and make changes:




The Miniserver's project is then opened on a new page:



    Now objects can be replaced, added, deleted or modified just like in an ordinary project. All deviations from the basic program are marked purple.



### Pair / replace Devices:

    The previously created placeholders are now replaced with the actual devices via the pairing dialogs of the respective Miniservers. This results in a different serial number as a deviation:




The deviations to the Multiplicator Project can be displayed in a window:




    Here a Miniserver can also be reset to Multiplicator default.


    A Miniserver can also be detached from the Multiplicator Project:



    After detaching, this Miniserver must also be deleted in the Multiplicator Project!




---


## Trusts


    [Trusts](https://www.loxone.com/help/trusts) can be used for user management, as well as sharing inputs, outputs or Intercoms accross the Miniservers of the Multiplicator Project.


    The trust manager must not be part of the Multiplicator Project, and all Miniservers must be authorized to join.
All local users can be replaced by trust users.