// Arduino code has been partially copied from AlexGyver <https://github.com/AlexGyver/Arduino_Ambilight>

//----------------------Settings-----------------------
#define NUM_LEDS 81
#define DI_PIN 13
#define OFF_TIME 60
#define CURRENT_LIMIT 2000

#define START_FLASHES 1
//----------------------Setting-----------------------

#define serialRate 115200
uint8_t prefix[] = {89, 124, 234}, hi, lo, chk, i;
#include <FastLED.h>
CRGB leds[NUM_LEDS];

void setup()
{
  FastLED.addLeds<WS2811, DI_PIN, RGB>(leds, NUM_LEDS);
  if (CURRENT_LIMIT > 0) FastLED.setMaxPowerInVoltsAndMilliamps(5, CURRENT_LIMIT);

  if (START_FLASHES) {
    LEDS.showColor(CRGB(41, 21, 6));
  }

  Serial.begin(serialRate);
  Serial.print("Ada\n");
}

void loop() {
  for (i = 0; i < sizeof prefix; ++i) {
waitLoop: while (!Serial.available());;
    if (prefix[i] == Serial.read()) continue;
    i = 0;
    goto waitLoop;
  }

  while (!Serial.available());;
  hi = Serial.read();
  while (!Serial.available());;
  lo = Serial.read();
  while (!Serial.available());;
  chk = Serial.read();
  if (chk != (hi ^ lo ^ 0x55))
  {
    i = 0;
    goto waitLoop;
  }

  memset(leds, 0, NUM_LEDS * sizeof(struct CRGB));
  for (int i = 0; i < NUM_LEDS; i++) {
    byte r, g, b;
    while (!Serial.available());
    r = Serial.read();
    while (!Serial.available());
    g = Serial.read();
    while (!Serial.available());
    b = Serial.read();
    leds[i].r = r;
    leds[i].g = g;
    leds[i].b = b;
  }
  FastLED.show();

  Serial.write(prefix[2]);
  Serial.write(prefix[1]);
  Serial.write(prefix[0]);
}
