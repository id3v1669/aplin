# aplin

headphones helper

## TODO

* implement sending packets to devices(name, case charging sound, Toggle Conversational Awareness)
* fix gui start on reconnect
* read & parse config

## Known bugs

* Airpods max do not switch on single mode to transparency
* Airpods ANC mode is not switchable when only one pod is active(same Firmware problem with airpods on macos too)

## Known bugs with workarounds

|Bug|Workaround|ToDo|
|---|---|---|
|on long standby airpods do not turn anc on and sound sometimes broken|Trigger device disconnect when airpods are not on|Base delay on config|

## Old bugs, fixed by bluez?

* on initial connection, codec switch back and forth required to make sound work

## Dev Notes

* waiting for bluest to be released for android, untill then crosplatform abandoned
* windows support is not planed as there is already solution and I don't use windows
