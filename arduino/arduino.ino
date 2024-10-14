// Arduino code has been partially copied from AlexGyver <https://github.com/AlexGyver/Arduino_Ambilight>

//----------------------Settings-----------------------
#define NUM_LEDS 81
#define DI_PIN 13
#define OFF_TIME 60
#define CURRENT_LIMIT 2000

#define START_FLASHES 1
//----------------------Setting-----------------------

int new_bright, new_bright_f;
unsigned long bright_timer, off_timer;

#define serialRate 115200
uint8_t prefix[] = {89, 124, 234}, hi, lo, chk, i;
#include <FastLED.h>
CRGB leds[NUM_LEDS];
boolean led_state = true;

void setup()
{
  FastLED.addLeds<WS2812, DI_PIN, GRB>(leds, NUM_LEDS);
  if (CURRENT_LIMIT > 0) FastLED.setMaxPowerInVoltsAndMilliamps(5, CURRENT_LIMIT);

  if (START_FLASHES) {
    // LEDS.showColor(CRGB(41, 21, 41));
    LEDS.showColor(CRGB(41, 21, 6));
  }

  Serial.begin(serialRate);
  Serial.print("Ada\n");
}

void check_connection() {
  if (led_state) {
    if (millis() - off_timer > (OFF_TIME * 1000)) {
      led_state = false;
      FastLED.clear();
      FastLED.show();
    }
  }
}

void loop() {
  if (!led_state) led_state = true;
  off_timer = millis();

  for (i = 0; i < sizeof prefix; ++i) {
waitLoop: while (!Serial.available()) check_connection();;
    if (prefix[i] == Serial.read()) continue;
    i = 0;
    goto waitLoop;
  }

  while (!Serial.available()) check_connection();;
  hi = Serial.read();
  while (!Serial.available()) check_connection();;
  lo = Serial.read();
  while (!Serial.available()) check_connection();;
  chk = Serial.read();
  if (chk != (hi ^ lo ^ 0x55))
  {
    i = 0;
    goto waitLoop;
  }

  memset(leds, 0, NUM_LEDS * sizeof(struct CRGB));
  for (int i = 0; i < NUM_LEDS; i++) {
    byte r, g, b;
    // читаем данные для каждого цвета
    while (!Serial.available()) check_connection();
    r = Serial.read();
    while (!Serial.available()) check_connection();
    g = Serial.read();
    while (!Serial.available()) check_connection();
    b = Serial.read();
    leds[i].r = r;
    leds[i].g = g;
    leds[i].b = b;
  }
  FastLED.show();  // записываем цвета в ленту
}
