// Code-Iot Arduino simulator: a tiny stand-in for an Arduino Uno that runs a
// sketch on a normal computer. Time is simulated (delay() is instant), the
// Serial Monitor is the program's output, Serial input comes from the
// playground's input box, and pins, PWM, tones, servos and the LCD are
// reported as text. A button on any input pin is "pressed" for 200 ms every
// 3 seconds. The run stops after 20 seconds of board time.
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include <cctype>
#include <cstdint>
#include <string>
#include <algorithm>

typedef uint8_t byte;
typedef uint16_t word;
typedef bool boolean;

#define HIGH 1
#define LOW 0
#define INPUT 0
#define OUTPUT 1
#define INPUT_PULLUP 2
#define LED_BUILTIN 13
#define A0 14
#define A1 15
#define A2 16
#define A3 17
#define A4 18
#define A5 19
#define CHANGE 1
#define FALLING 2
#define RISING 3
#define DEC 10
#define HEX 16
#define OCT 8
#define BIN 2
#define PROGMEM
#define F(s) (s)
#define PI 3.1415926535897932384626433832795
#define TWO_PI 6.283185307179586476925286766559
#define DEG_TO_RAD 0.017453292519943295769236907684886
#define RAD_TO_DEG 57.295779513082320876798154814105
#define sq(x) ((x) * (x))
#define bitRead(v, b) (((v) >> (b)) & 1)
#define bitSet(v, b) ((v) |= (1UL << (b)))
#define bitClear(v, b) ((v) &= ~(1UL << (b)))
#define bitWrite(v, b, x) ((x) ? bitSet(v, b) : bitClear(v, b))
#define bit(b) (1UL << (b))
#define lowByte(w) ((uint8_t)((w) & 0xff))
#define highByte(w) ((uint8_t)((w) >> 8))
using std::min;
using std::max;
using std::isnan;
using std::isinf;

// ---------- simulated time ----------
static unsigned long __sim_us = 0;
static const unsigned long __SIM_LIMIT_US = 20000000UL;
static long __sim_out = 0;
static int __pin_mode[20];
static int __pin_val[20];
static int __pwm_val[20];
static unsigned long __pwm_said[20];
static void (*__isr[2])() = {nullptr, nullptr};
static int __isr_mode[2];

static void __end(const char *why) {
  std::printf("\n[simulation stopped: %s]\n", why);
  std::fflush(stdout);
  std::_Exit(0);
}
static bool __pressed_at(unsigned long us) { return (us / 1000) % 3000 >= 1000 && (us / 1000) % 3000 < 1200; }
static void __advance(unsigned long us) {
  bool before = __pressed_at(__sim_us);
  __sim_us += us;
  bool after = __pressed_at(__sim_us);
  if (!before && after) {          // button goes down: FALLING / CHANGE
    for (int i = 0; i < 2; i++) if (__isr[i] && __isr_mode[i] != RISING) __isr[i]();
  } else if (before && !after) {   // button released: RISING / CHANGE
    for (int i = 0; i < 2; i++) if (__isr[i] && __isr_mode[i] != FALLING) __isr[i]();
  }
  if (__sim_us > __SIM_LIMIT_US) __end("20 seconds of board time");
}
static void __stamp() { std::printf("[%6.3f s] ", __sim_us / 1e6); }
static void __out(const char *s) {
  __sim_out += (long)std::strlen(s);
  std::fputs(s, stdout);
  if (__sim_out > 30000) __end("too much output");
}

unsigned long millis() { __advance(50); return __sim_us / 1000; }
unsigned long micros() { __advance(50); return __sim_us; }
void delay(unsigned long ms) { __advance(ms * 1000UL); }
void delayMicroseconds(unsigned int us) { __advance(us); }

static const char *__pin_name(int pin) {
  static char buf[8];
  if (pin >= A0 && pin <= A5) std::snprintf(buf, sizeof buf, "A%d", pin - A0);
  else std::snprintf(buf, sizeof buf, "%d", pin);
  return buf;
}
void pinMode(int pin, int mode) { if (pin >= 0 && pin < 20) __pin_mode[pin] = mode; }
void digitalWrite(int pin, int v) {
  v = v ? 1 : 0;
  if (pin < 0 || pin >= 20) return;
  if (__pin_mode[pin] != OUTPUT) { __pin_mode[pin] = v ? INPUT_PULLUP : INPUT; return; }
  if (__pin_val[pin] == v) return;
  __pin_val[pin] = v;
  __stamp(); std::printf("pin %s -> %s\n", __pin_name(pin), v ? "HIGH" : "LOW");
  __sim_out += 30;
}
int digitalRead(int pin) {
  __advance(20);
  if (pin < 0 || pin >= 20) return LOW;
  if (__pin_mode[pin] == OUTPUT) return __pin_val[pin];
  bool pressed = __pressed_at(__sim_us);
  if (__pin_mode[pin] == INPUT_PULLUP) return pressed ? LOW : HIGH;
  return pressed ? HIGH : LOW;
}
int analogRead(int pin) {
  __advance(110);
  double t = __sim_us / 1e6;
  int v = (int)(512 + 400 * std::sin(t / 2.0 + pin));
  return std::max(0, std::min(1023, v));
}
void analogWrite(int pin, int v) {
  if (pin < 0 || pin >= 20) return;
  v = std::max(0, std::min(255, v));
  if (__pin_mode[pin] != OUTPUT) __pin_mode[pin] = OUTPUT;
  if (__pwm_val[pin] == v) return;
  __pwm_val[pin] = v;
  if (__sim_us - __pwm_said[pin] < 500000UL && __pwm_said[pin] != 0) return;  // keep the log readable
  __pwm_said[pin] = __sim_us ? __sim_us : 1;
  __stamp(); std::printf("pin %s PWM %d/255\n", __pin_name(pin), v);
  __sim_out += 30;
}
void tone(int pin, unsigned int freq, unsigned long dur = 0) {
  __stamp();
  if (dur) std::printf("pin %d tone %u Hz for %lu ms\n", pin, freq, dur);
  else std::printf("pin %d tone %u Hz\n", pin, freq);
  __sim_out += 30;
}
void noTone(int pin) { __stamp(); std::printf("pin %d tone off\n", pin); }
unsigned long pulseIn(int, int, unsigned long timeout = 1000000UL) {
  double t = __sim_us / 1e6;
  unsigned long us = (unsigned long)(5800 + 4500 * std::sin(t / 3.0));  // 20..180 cm
  if (us > timeout) { __advance(timeout); return 0; }
  __advance(us);
  return us;
}
long map(long x, long a, long b, long c, long d) { return (x - a) * (d - c) / (b - a) + c; }
template <class T, class L, class H> auto constrain(T x, L lo, H hi) -> decltype(x + lo + hi) { return x < lo ? lo : (x > hi ? hi : x); }
long random(long hi) { return hi > 0 ? std::rand() % hi : 0; }
long random(long lo, long hi) { return hi > lo ? lo + random(hi - lo) : lo; }
void randomSeed(unsigned long s) { std::srand((unsigned)s); }
int digitalPinToInterrupt(int p) { return p == 2 ? 0 : (p == 3 ? 1 : -1); }
void attachInterrupt(int n, void (*f)(), int mode) { if (n == 0 || n == 1) { __isr[n] = f; __isr_mode[n] = mode; } }
void detachInterrupt(int n) { if (n == 0 || n == 1) __isr[n] = nullptr; }
void noInterrupts() {}
void interrupts() {}
bool isDigit(int c) { return std::isdigit(c); }
bool isAlpha(int c) { return std::isalpha(c); }
bool isAlphaNumeric(int c) { return std::isalnum(c); }
bool isSpace(int c) { return std::isspace(c); }
bool isWhitespace(int c) { return c == ' ' || c == '\t'; }
bool isUpperCase(int c) { return std::isupper(c); }
bool isLowerCase(int c) { return std::islower(c); }
bool isPunct(int c) { return std::ispunct(c); }
uint16_t pgm_read_word(const void *p) { return *(const uint16_t *)p; }
uint8_t pgm_read_byte(const void *p) { return *(const uint8_t *)p; }
int strlen_P(const char *s) { return (int)std::strlen(s); }
char *strcpy_P(char *d, const char *s) { return std::strcpy(d, s); }

// ---------- String ----------
class String {
  std::string s;
 public:
  String() {}
  String(const char *c) : s(c ? c : "") {}
  String(const std::string &x) : s(x) {}
  String(char c) : s(1, c) {}
  String(int v, int base = 10) { from(v, base); }
  String(unsigned int v, int base = 10) { from(v, base); }
  String(long v, int base = 10) { from(v, base); }
  String(unsigned long v, int base = 10) { from((long)v, base); }
  String(double v, int digits = 2) { char b[64]; std::snprintf(b, sizeof b, "%.*f", digits, v); s = b; }
  void from(long v, int base) {
    if (base == 10) { s = std::to_string(v); return; }
    unsigned long u = (unsigned long)v; std::string r;
    do { int d = u % base; r += (char)(d < 10 ? '0' + d : 'A' + d - 10); u /= base; } while (u);
    s.assign(r.rbegin(), r.rend());
  }
  unsigned int length() const { return (unsigned)s.size(); }
  char charAt(unsigned int i) const { return i < s.size() ? s[i] : 0; }
  void setCharAt(unsigned int i, char c) { if (i < s.size()) s[i] = c; }
  char operator[](unsigned int i) const { return charAt(i); }
  const char *c_str() const { return s.c_str(); }
  long toInt() const { return std::atol(s.c_str()); }
  float toFloat() const { return (float)std::atof(s.c_str()); }
  void trim() {
    size_t a = s.find_first_not_of(" \t\r\n");
    size_t b = s.find_last_not_of(" \t\r\n");
    s = a == std::string::npos ? "" : s.substr(a, b - a + 1);
  }
  void toUpperCase() { for (auto &c : s) c = (char)std::toupper(c); }
  void toLowerCase() { for (auto &c : s) c = (char)std::tolower(c); }
  int indexOf(char c, unsigned int from = 0) const { auto p = s.find(c, from); return p == std::string::npos ? -1 : (int)p; }
  int indexOf(const String &t, unsigned int from = 0) const { auto p = s.find(t.s, from); return p == std::string::npos ? -1 : (int)p; }
  int lastIndexOf(char c) const { auto p = s.rfind(c); return p == std::string::npos ? -1 : (int)p; }
  String substring(unsigned int a) const { return a >= s.size() ? String() : String(s.substr(a)); }
  String substring(unsigned int a, unsigned int b) const {
    if (a > b) std::swap(a, b);
    if (a >= s.size()) return String();
    return String(s.substr(a, std::min<size_t>(b, s.size()) - a));
  }
  bool startsWith(const String &p) const { return s.rfind(p.s, 0) == 0; }
  bool endsWith(const String &p) const { return s.size() >= p.s.size() && s.compare(s.size() - p.s.size(), p.s.size(), p.s) == 0; }
  bool equals(const String &o) const { return s == o.s; }
  bool equalsIgnoreCase(const String &o) const {
    if (s.size() != o.s.size()) return false;
    for (size_t i = 0; i < s.size(); i++) if (std::tolower(s[i]) != std::tolower(o.s[i])) return false;
    return true;
  }
  void replace(const String &a, const String &b) {
    if (a.s.empty()) return;
    size_t p = 0;
    while ((p = s.find(a.s, p)) != std::string::npos) { s.replace(p, a.s.size(), b.s); p += b.s.size(); }
  }
  void remove(unsigned int i, unsigned int n = 1) { if (i < s.size()) s.erase(i, n); }
  bool concat(const String &o) { s += o.s; return true; }
  String &operator+=(const String &o) { s += o.s; return *this; }
  String &operator+=(const char *o) { s += o; return *this; }
  String &operator+=(char c) { s += c; return *this; }
  String &operator+=(int v) { s += std::to_string(v); return *this; }
  friend String operator+(const String &a, const String &b) { return String(a.s + b.s); }
  friend String operator+(const String &a, const char *b) { return String(a.s + b); }
  friend String operator+(const char *a, const String &b) { return String(a + b.s); }
  friend String operator+(const String &a, char c) { return String(a.s + c); }
  friend String operator+(const String &a, int v) { return String(a.s + std::to_string(v)); }
  friend String operator+(const String &a, long v) { return String(a.s + std::to_string(v)); }
  friend String operator+(const String &a, unsigned long v) { return String(a.s + std::to_string(v)); }
  friend String operator+(const String &a, double v) { return a + String(v); }
  bool operator==(const String &o) const { return s == o.s; }
  bool operator==(const char *o) const { return s == o; }
  bool operator!=(const String &o) const { return s != o.s; }
  bool operator!=(const char *o) const { return s != o; }
  bool operator<(const String &o) const { return s < o.s; }
};

// ---------- Print / Serial ----------
class Print {
 public:
  virtual void emit(const char *s) { __out(s); }
  virtual ~Print() {}
  size_t print(const char *v) { emit(v); return std::strlen(v); }
  size_t print(const String &v) { return print(v.c_str()); }
  size_t print(char c) { char b[2] = {c, 0}; return print(b); }
  size_t print(unsigned char v, int base = DEC) { return print((unsigned long)v, base); }
  size_t print(int v, int base = DEC) { return base == DEC ? print((long)v) : print((unsigned long)(unsigned int)v, base); }
  size_t print(unsigned int v, int base = DEC) { return print((unsigned long)v, base); }
  size_t print(long v, int base = DEC) {
    if (base != DEC) return print((unsigned long)v, base);
    char b[32]; std::snprintf(b, sizeof b, "%ld", v); return print(b);
  }
  size_t print(unsigned long v, int base = DEC) { return print(String((long)v, base)); }
  size_t print(double v, int digits = 2) { char b[64]; std::snprintf(b, sizeof b, "%.*f", digits, v); return print(b); }
  size_t print(bool v) { return print((int)v); }
  size_t println() { return print("\n"); }
  template <class T> size_t println(const T &v) { size_t n = print(v); println(); return n + 1; }
  template <class T> size_t println(const T &v, int f) { size_t n = print(v, f); println(); return n + 1; }
  size_t write(uint8_t c) { return print((char)c); }
};

class SerialClass : public Print {
  std::string in;
  size_t pos = 0;
  bool loaded = false;
  unsigned long timeout = 1000;
  void load() {
    if (loaded) return;
    loaded = true;
    int c;
    while ((c = std::getchar()) != EOF) in += (char)c;
  }
 public:
  void begin(unsigned long) { load(); }
  void end() {}
  void flush() { std::fflush(stdout); }
  int available() { load(); __advance(20); return (int)(in.size() - pos); }
  int read() { load(); return pos < in.size() ? (unsigned char)in[pos++] : -1; }
  int peek() { load(); return pos < in.size() ? (unsigned char)in[pos] : -1; }
  void setTimeout(unsigned long ms) { timeout = ms; }
  String readStringUntil(char t) {
    load(); std::string r;
    while (pos < in.size() && in[pos] != t) r += in[pos++];
    if (pos < in.size()) pos++;
    if (pos >= in.size()) __advance(timeout * 1000UL);
    return String(r);
  }
  String readString() { load(); std::string r = in.substr(pos); pos = in.size(); __advance(timeout * 1000UL); return String(r); }
  size_t readBytesUntil(char t, char *buf, size_t n) {
    load(); size_t k = 0;
    while (pos < in.size() && in[pos] != t && k < n) buf[k++] = in[pos++];
    if (pos < in.size() && in[pos] == t) pos++;
    return k;
  }
  long parseInt() {
    load();
    while (pos < in.size() && !std::isdigit((unsigned char)in[pos]) && in[pos] != '-') pos++;
    if (pos >= in.size()) { __advance(timeout * 1000UL); return 0; }
    long sign = 1; if (in[pos] == '-') { sign = -1; pos++; }
    long v = 0; while (pos < in.size() && std::isdigit((unsigned char)in[pos])) v = v * 10 + (in[pos++] - '0');
    return sign * v;
  }
  float parseFloat() {
    load();
    while (pos < in.size() && !std::isdigit((unsigned char)in[pos]) && in[pos] != '-' && in[pos] != '.') pos++;
    if (pos >= in.size()) { __advance(timeout * 1000UL); return 0; }
    size_t used = 0; float v = std::stof(in.substr(pos), &used); pos += used; return v;
  }
  explicit operator bool() const { return true; }
};
SerialClass Serial;

// ---------- libraries ----------
class Servo {
  int pin = -1, angle = -1;
  unsigned long said = 0;
 public:
  uint8_t attach(int p, int = 544, int = 2400) { pin = p; return 1; }
  void detach() { pin = -1; }
  bool attached() { return pin >= 0; }
  void write(int a) {
    a = std::max(0, std::min(180, a));
    if (a == angle) return;
    angle = a;
    if (said && __sim_us - said < 300000UL) return;
    said = __sim_us ? __sim_us : 1;
    __stamp(); std::printf("servo on pin %d -> %d deg\n", pin, a); __sim_out += 30;
  }
  void writeMicroseconds(int us) { write((int)map(us, 544, 2400, 0, 180)); }
  int read() { return angle; }
};

class TwoWire {
  int addr = 0;
 public:
  void begin() {}
  void begin(int) {}
  void setClock(unsigned long) {}
  void beginTransmission(int a) { addr = a; }
  uint8_t endTransmission(bool = true) { return (addr == 0x27 || addr == 0x3F || addr == 0x68) ? 0 : 2; }
  size_t write(uint8_t) { return 1; }
  uint8_t requestFrom(int, int n) { return (uint8_t)n; }
  int available() { return 1; }
  int read() { return 0; }
};
TwoWire Wire;

class LiquidCrystal_I2C : public Print {
  int cols, rows, cx = 0, cy = 0;
  char buf[4][41];
  void show() {
    __stamp(); std::printf("LCD");
    for (int r = 0; r < rows; r++) std::printf(" |%.*s|", cols, buf[r]);
    std::printf("\n"); __sim_out += 40;
  }
 public:
  LiquidCrystal_I2C(uint8_t, int c, int r) : cols(std::min(c, 40)), rows(std::min(r, 4)) { clear(); }
  void init() {}
  void begin(int = 16, int = 2) {}
  void backlight() {}
  void noBacklight() {}
  void clear() { for (int r = 0; r < 4; r++) { std::memset(buf[r], ' ', 40); buf[r][40] = 0; } cx = cy = 0; }
  void home() { cx = cy = 0; }
  void setCursor(int c, int r) { cx = c; cy = r; }
  void emit(const char *s) override {
    for (; *s; s++) {
      if (*s == '\n' || *s == '\r') continue;
      if (cy >= 0 && cy < rows && cx >= 0 && cx < cols) buf[cy][cx] = *s;
      cx++;
    }
    show();
  }
};

#define DHT11 11
#define DHT22 22
class DHT {
 public:
  DHT(uint8_t, uint8_t) {}
  void begin() {}
  float readTemperature(bool f = false) { float c = 23.4f + 1.5f * (float)std::sin(__sim_us / 7e6); __advance(2000); return f ? c * 9 / 5 + 32 : c; }
  float readHumidity() { __advance(2000); return 41.0f + 3.0f * (float)std::sin(__sim_us / 9e6); }
  float computeHeatIndex(float t, float, bool = true) { return t; }
};

class EEPROMClass {
  uint8_t mem[1024];
 public:
  EEPROMClass() { std::memset(mem, 0xFF, sizeof mem); }
  uint8_t read(int a) { return mem[a & 1023]; }
  void write(int a, uint8_t v) { mem[a & 1023] = v; }
  void update(int a, uint8_t v) { mem[a & 1023] = v; }
  uint16_t length() { return 1024; }
  template <class T> T &get(int a, T &t) { std::memcpy(&t, mem + (a & 1023), sizeof(T)); return t; }
  template <class T> const T &put(int a, const T &t) {
    std::memcpy(mem + (a & 1023), &t, sizeof(T));
    __stamp(); std::printf("EEPROM: saved %d byte(s) at address %d\n", (int)sizeof(T), a); __sim_out += 40;
    return t;
  }
  uint8_t &operator[](int a) { return mem[a & 1023]; }
};
EEPROMClass EEPROM;

class SPIClass {
 public:
  void begin() {}
  void end() {}
  uint8_t transfer(uint8_t v) { return v; }
};
SPIClass SPI;
