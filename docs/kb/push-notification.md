# Push Notification

Source: https://www.loxone.com/enen/kb/push-notification/

---

Notifications can be used in Loxone Config for a wide range of applications. When sent, they are displayed in the Loxone App and Web Interface.
You can specify when a notification is to be sent and what text is to be displayed.


    On Android and iOS mobile devices, push notifications are also displayed outside of the Loxone app.
The Miniserver must be registered and connected to the Internet for this feature. Up to 200 push notifications can be sent per day.



|  | Push notifications are not available on new Huawei smartphones/tablets, as these depend on Google services, which are no longer included in Huawei devices due to the US embargo. |
| --- | --- |


## Table of Contents
- [Programming example](#baseconf)
- [Display of the notification](#visu)
- [Push Notifications: Registered devices](#RegDev)
- [Sound of the notification](#SoundPush)




---


## Programming example


    First, a new notification is created under Messages:



    In the following example we name the notification Garage, and drag the object to the programming page.
We can now connect this output connector to the output Op of the Garage/ Gate function block.
Then we define the text in the settings of the output connector, which will be sent when the input value is 1.
We can select which users or groups receive the notification by setting the appropriate permissions.
In this example, the selected users will receive a notification every time the gate is opened:




---


## Display of the notification


    When triggered, the notification is displayed in the App and Web Interface:



    Outside of the App, the notification is displayed as a push notification, the following example shows it on an Android device:



    When the App connects to the Miniserver for the first time, the user is prompted to enable Push Notifications.
It is also possible to enable or disable receiving notifications in the App's settings.



---


## Push Notifications: Registered devices


    Under "Push Notifications: Registered devices", all devices with push notifications enabled in the App are listed with the associated user.


    Devices that no longer exist can be removed using the button "Unregister device":




---


## Sound of the notification


    Since the release of Config & App 14.5, different types of notifications now feature unique sound effects, making it easier to distinguish between them.


    Critical or time-sensitive push notifications—such as burglar alarms, fire alarms, or intercom rings—can now bypass Focus Mode on both Android and iOS, ensuring they are delivered promptly even when Focus Mode is active.


    **Android:**


    Navigate to Settings - Notifications - App notifications - Loxone - Notification Categories to view the different categories, such as Alert, Default, and Doorbell.



    **Note:** If you don't see the option "Notification Categories", please follow [this](https://www.androidpolice.com/samsung-disables-notification-channels-on-all-one-ui-61-devices/) guide.


    Selecting "Doorbell" for example, shows that the sound is provided by the app and can be customized. However, once this sound is changed, restoring the default app-provided sound requires reinstalling the app.



    **iOS:**


    Navigate to Settings - Notifications - Loxone.