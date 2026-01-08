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

## compatibility

this client has been verified to work on windows. macOS/linux support haven't been verified yet, but they should work fine if all the dependencies are installed.

## compiling

### requirements
- Qt6 installed
- libcurl installed (you can install this by just having curl i think? if not install libcurl through your package manager)
- CMake
- Ninja
- c++ compiler (mingw-g++ for windows, g++ should work on linux, i have no idea about macOS)

if you're on windows, you will need to install all of these using msys2

clone the repo, then cd into it and do

    mkdir build
    cd build

now do 

    cmake -S .. -B . -G Ninja

this will generate the build files in `build/`. now you can build!

    cmake --build .

the final executable will be located in the `build/` directory you created.