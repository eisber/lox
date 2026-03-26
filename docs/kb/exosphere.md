# Exosphere

Source: https://www.loxone.com/enen/kb/exosphere/

---

Exosphere offers an optional extension providing an additional level of organization and analysis in intelligent building management. This online solution is specifically designed to simplify the operation and maintenance of Miniserver fleets, whether individually or as part of a team.


    Exosphere enables centralized storage and display of shared building data. Data from complex and distributed Loxone installations can be efficiently visualized in system diagrams or dashboards and compared with each other.


    At Loxone, the privacy of our customers is always a top priority. Therefore, all data sharing with Exosphere is done on an opt-in basis to guarantee you full control and security.



|  | Loxone Exosphere requires at least Config version 15.2.9.30 and a current Miniserver generation with Remote Connect.A Miniserver can be added to a maximum of 10 workspaces. |
| --- | --- |


## Table of Contents
- [Initial Setup](#setup)
- [Create and manage databases](#createdb)
- [Sending Reports via Email or Webhook](#emailwebhook)
- [Object-Level Permissions](#permissions)
- [Inputs, Outputs, Properties](#Diagnostic)




---


## Initial Setup


    Open [Exosphere](https://exo.loxone.com/), log in with your Loxone account and create or join a workspace:



    Depending on the version of the Miniserver and Loxone Config, there are 2 ways to add a Miniserver to a workspace:




### Version 15.2.9.30 or higher

    In the next window, a provisioning code is created, which must be saved to the Miniserver. To do so, copy the code.



    Now switch to Loxone Config and add Exosphere to Network Periphery:



    In the properties window, a suitable Exo user can be selected and the copied provisioning code pasted:



    Then save to the Miniserver.



---


## Create and manage databases


    In Exosphere, databases can be created, configured, and visualized. You can either upload an existing database (.json) or define the columns directly in Exosphere:



    The database can then be exported. For the import into the Config, select "Export Schema":



    Switch to Loxone Config and click on "Insert database table":



    In the properties window, the "Database Connector Table Configuration" can be opened. Here, an Exosphere database can either be imported as a file or, if logged into Exosphere in the Config, imported directly from Exosphere.


    The table ID is taken from the imported file.



    Alternatively, columns can be added in the configuration, but they must then be added in Exosphere as well.



|  | Ensure that the table ID and column ID in the configuration match the IDs in Exosphere, as otherwise, no connection will be established. |
| --- | --- |

    In the configuration window, it can be defined when the data should be written to the database:



    Now, drag the "Database Connector Table" onto the programming page and connect it via the "API Connector" to the [Session Database Connector](https://www.loxone.com/help/SessionDatabaseConnector/) or [Event Database Connector](https://www.loxone.com/help/EventDatabaseConnector/) to populate the database.



---


## Sending Reports via Email or Webhook


    Reports can be configured via the edit icon to be sent automatically via email and/or webhook.
You can define the data content, format, and the desired delivery interval individually.




|  | The Exosphere Miniserver Enterprise supports the delivery of up to 10 reports per database.With the Pro version, a single report can be sent via email. |
| --- | --- |


---


## Object-Level Permissions


    Charts, dashboards, and databases can now be shared individually – without requiring full system access:




|  | When sharing with the "Anyone with the link" option, no Loxone account is required to access the shared object. |
| --- | --- |


---


## Diagnostic Inputs




| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Exosphere | Indicates whether the device can be reached by the Miniserver.Diagnostics for Air devicesDiagnostics for Tree devicesDiagnostics for Extensions | Digital | 0/1 |








---


## Properties




| Summary | Description | Default Value |
| --- | --- | --- |
| Provisioning Code | Provisioning Code to add this Miniserver to Exosphere | - |
| Monitor service | If checked, you will be notified by the System Status or Cloud Mailer if this service is no longer available or offline. | - |
| User | The plugin gains access to the same user interface as the selected user | - |








---