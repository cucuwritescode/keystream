keystream
=========

plays sounds while you type on your keyboard


DESCRIPTION
-----------

keystream converts typing into pitched sine tones. each keypress
triggers a note. the daemon runs silently in the background,
synthesising audio in real time.

the system operates with 32 concurrent voices, recursive sine
oscillators, and sub-millisecond latency. no external dependencies
at runtime.


REQUIREMENTS
------------

```
macos       14.0 or later
terminal    accessibility permission required
```

rust is only needed when building from source. binary installs have no
prerequisites beyond macos itself.


INSTALLATION
------------

### homebrew (recommended)

```bash
brew tap cucuwritescode/keystream
brew install keystream
```

### installer (.pkg)

download the `.pkg` from the
[latest release](https://github.com/cucuwritescode/keystream/releases/latest).
double-click to install. accessibility settings will open automatically
after installation.

### direct download

```bash
curl -sSL https://raw.githubusercontent.com/cucuwritescode/keystream/main/scripts/install.sh | sh
```

### cargo

```bash
cargo install keystream-audio
```

the binary installs as `keystream`.

### from source

```bash
git clone https://github.com/cucuwritescode/keystream
cd keystream
make install
```

### daemon (launchd)

to run keystream as a background service:

```bash
make install-daemon
```

to stop and remove the daemon:

```bash
make uninstall-daemon
```


USAGE
-----

```bash
keystream start         # initiate daemon
keystream stop          # terminate daemon
keystream status        # query state
keystream run           # foreground mode
```

modes are selected at startup:

```
pentatonic              C D E G A
lydian                  C D E F# G A B
```

example:

```
$ keystream start

KEYSTREAM 0.1
--------------

select mode
> pentatonic   C D E G A
  lydian       C D E F# G A B

mode      : pentatonic
scale     : C D E G A
voices    : 32

starting daemon...
attaching keyboard stream...
opening audio device...

ONLINE
```


DESIGN
------

```
voices          32 concurrent
oscillator      recursive sine (no trig in audio thread)
envelope        5ms attack, 200ms release
sample rate     system default
latency         < 1ms

letters         melody (pentatonic/lydian mapping)
numbers         high register accents
punctuation     harmonic gestures
modifiers       chords and drones
```


ARCHITECTURE
------------

```
src/
├── main.rs         daemon management, signal handling
├── audio.rs        cpal output, voice mixing
├── voice.rs        recursive oscillator, envelope
├── keyboard.rs     device_query event capture
└── mapping.rs      key to pitch translation
```


UNINSTALL
---------

if installed via script or direct download:

```bash
curl -sSL https://raw.githubusercontent.com/cucuwritescode/keystream/main/scripts/uninstall.sh | sh
```

if installed from source:

```bash
make uninstall
```


LICENCE
-------

MIT
