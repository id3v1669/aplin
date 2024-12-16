# aplin
headphones helper

# TODO:
* delete device events from monitoring if device is not availible under adapter
* refcator code
* implement sending packets to devices(name, case charging sound, Toggle Conversational Awareness)
* fix gui start on reconnect
* read & parse config

# Known bugs
Most of them are problems with airpods in general and not with this software, but potentialy can be fixed.
* interface doesnt always restart on airpods reconnect
* on airpods reconnect mode stays off and no sound
* on initial connection, codec switch back and forth required to make sound work
* on standby mode turns to off and on airpods max only left ear works untill reinit (reinit doesn't always happen)


# Notes
* waiting for bluest to be released for android, untill then crosplatform abandoned
* windows support is not planed as there is already solution and I don't use windows
