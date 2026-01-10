# twtGUI

twtGUI is a graphical client for twtxt.

made this since i was feeling too lazy to do 
    twtxt timeline
in my terminal (and i also got too lazy to further develop the old version)

made with C++ and Qt

## features

- post tweets 
- view timeline
- view other twtxt.txts
- manage who you follow
- tweak some settings
    - nickname
    - twtxt.txt filepath
    - url to your public twtxt.txt
    - toggle whether the names in the time will be colored or not

## installing

you can download the latest version [here.](https://github.com/taxevaiden/twtGUI/releases)

`twtGUI-windows-x64.zip` has all of the dependencies inside by default and should work just fine, however `twtGUI-linux-x64.zip` (built using ubuntu) is different

for linux, you will need

- Qt 6.10 installed (i think you may have to manually compile it yourself as from my experience `apt` has an older version of qt6)
- curl installed

### compatibility

this client has been verified to work on windows and linux. macOS hasn't been verified yet but it should work if all the dependencies are installed correctly.

## compiling

### prerequisites
- Qt 6.10
- libcurl installed (you can install this by just having curl i think? if not install libcurl through your package manager)
- CMake
- Ninja
- c++ compiler (mingw-g++ for windows, g++ should work on linux, i have no idea about macOS)

if you're on windows, you will need to install all of these using msys2 and compile using msys2 (i used the mingw64 shell):
- `mingw-w64-x86_64-qt6-base` (Qt6)
- `mingw-w64-x86_64-curl` (curl)
- `mingw-w64-x86_64-cmake` (CMake)
- `mingw-w64-x86_64-ninja` (Ninja)
- `mingw-w64-x86_64-toolchain` (gcc and g++)

clone the repo, then cd into it and do

    mkdir build
    cd build

now do 

    cmake -S .. -B . -G Ninja

this will generate the build files in `build/`. now you can build!

    cmake --build .

the final executable will be located in the `build/` directory you created
