# Minesweeper
<p align="center"><img src="/docs/ms.png"  width="556"></p>

This is a little minesweeper game that I've been working on, written in Rust using the [Macroquad](https://github.com/not-fl3/macroquad) framework.
 
 There's a web build, plus some lovely juicy details about how some of it works, on [my website](https://jumbledfox.github.io/minesweeper)!

## Controls
|Action|Control|
|--|--|
|Dig|Left click|
|Flag|Right click|
|Chord|Middle click, or Right+Left click|

## Info
It's built on top of my own implementation of an immediate-mode GUI. It isn't the most efficient, but it works very well for my purposes. Casey Muratori's [talk on this subject](https://youtu.be/Z1qyvQsjK5Y) was invaluable to me when implementing my own!

Below you can see some of the GUI,  namely the menubar and a popup window.
<p align="center"><img src="/docs/menubar_popup.png" width="509"></p>

There's a very cool circular explosion effect which I'm quite proud of.   
(this would've been SO much nicer as an mp4 file, i.e. improved video quality, smaller file size, and sound, but silly github doesn't like putting videos in markdown files for some reason!!!)
<p align="center"><img src="/docs/explosions.gif" width="409"></p>

You can also make custom games!
<p align="center"><img src="/docs/custom.png" width="475"></p>

##  Credits
Macroquad - [not-fl3](https://github.com/not-fl3/macroquad)   
WASM build script ([build_wasm.sh](build_wasm.sh)) - [Tom Solberg (and more!)](https://gist.github.com/nicolas-sabbatini/8af10dddc96be76d2bf24fc671131add)   
MP4 to GIF script - [troyane](https://gist.github.com/troyane/af2a43a9eecda6d3de7729154cc4fe5e)   
Explosion sound - [AyeDrevis](https://freesound.org/people/AyaDrevis/sounds/649191/)   
Lowercase character 'e' - [04b03](https://www.dafont.com/04b-03.font)   

Everything else made by me, [jumbledFox](https://jumbledfox.github.io) :3   
