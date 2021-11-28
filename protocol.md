# Bryggio protocol

_WIP: Just a place to collect thoughts for now._

## Messages

- All messages have mandatory field *timestamp* which is a UNIX timestamp, timed at message creation.
  Time between message creation and publication should be neglible.
- All messages (with a recipient) have mandatory field *id*
