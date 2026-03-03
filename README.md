# MQTT-driven Virtual Volume Knob

There's a 99% chance this repository is not useful to you, just a heads up.

If you have a Home Assistant compatible amp/reciever but it doesn't have its own volume display for some reason, this
project lets you use an esp32 with an LCD as a virtual volume display for it.

It displays volume, current source, and power.

It's set up for an esp32c6 MCU + GC9A01 LCD at the moment, but the display is abstracted through mipidsi so it's
fairly straightforward to change either of the above.

![example image](example.png)

---

#### Prerequisites

You'll also need to set up the other side of it inside Home Assistant via an automation.

Something along the lines of this (your entity & device IDs will vary)

```yaml
alias: Publish Amp State Changes to MQTT
triggers:
  - trigger: state
    entity_id:
      - media_player.wxa50
    attribute: volume_level
  - trigger: state
    entity_id:
      - media_player.wxa50
    attribute: source
  - type: changed_states
    device_id: aaaaaaaaaaaaaaaaa
    entity_id: bbbbbbbbbbbbbbbbb
    domain: media_player
    trigger: device
conditions: []
actions:
  - action: mqtt.publish
    metadata: {}
    data:
      evaluate_payload: false
      qos: 1
      retain: true
      topic: /amplifier/wxa-50/volume
      payload: >-
        {% set volume = state_attr('media_player.wxa50', 'volume_level') %}
        {% if volume is defined and volume != None and volume != "None" and volume != "" %}
          {{ volume | float * 100 }}
        {% else %}
          0
        {% endif %}
  - action: mqtt.publish
    metadata: {}
    data:
      evaluate_payload: false
      qos: 1
      retain: true
      topic: /amplifier/wxa-50/power
      payload: >-
        {{ 'ON' if states('media_player.wxa50') != 'off' else 'OFF' }}
  - action: mqtt.publish
    metadata: {}
    data:
      evaluate_payload: false
      qos: 1
      retain: true
      topic: /amplifier/wxa-50/input
      payload: >-
        {{ state_attr('media_player.wxa50', 'source') if state_attr('media_player.wxa50', 'source') != None else 'unknown' }}
mode: single
```
