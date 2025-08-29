# rustrhythm
An experimental stepmania map parser written in Rust. It runs on an SDL2 front.
This program runs .sm-files and displays them similarly to how stepmania would do it.

## Motivation
Once when playing around with stepmania, I took a look at the .sm files and thought "hey that looks pretty nice to parse".
I finally bit the bullet and gave it a shot using Rust to learn more about both stepmania maps and rust programming.
This program has been tested with a few maps I have on my disk so no guarantee everything works alright.

## Features
- Single notes
- Long notes
- Menu sample parsing
- Multiple difficulties
- Point system (not close to stepmania's)

## Features missing
- Mines (I personally don't like them)
- Video backgrounds
- Stops (they are skipped)
- HP (the main point is to run the map)

In case someone really wants to contribute and fix those features missing features.
This isn't meant to replace stepmania. It just displays the maps and you can sort of play them.