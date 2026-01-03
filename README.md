# twtGUI

twtGUI is a graphical client for twtxt.

made this since i was feeling too lazy to do 
    twtxt timeline
in my terminal (and i also got too lazy to further develop the old version)

made with C++ and Qt

## requirements

- windows
- a twtxt config file and a twtxt.txt (you can get both of these if you install twtxt)

## features

right now, you can post tweets and see tweets from other people.

## compatibility

the client assumes you're on windows so it might not work for those who are on macOS/linux. works perfectly fine on windows though!

## compiling

you'll need 
- Qt6 installed (i think)
- CMake
- Ninja
- some c++ compiler like g++ (included with Qt6 installation at least for windows)

clone the repo, then cd into it and do

    mkdir build
    cd build

now do 

    cmake -S .. -B .

this will generate the build files in `build/`. now you can build!

    cmake --build .

the final executable will be located in the `build/` directory you created.