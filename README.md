# Xaver
This tool attempts to implement saving and loading for Exanima, similar to the Salvus save manager by steam user Silk, a link to which you can find at (https://steamcommunity.com/app/362490/discussions/0/4032473829604421294/).



**As of right now, Xaver supports the following Operating Systems:**
* Windows
* Linux

## Features
Along with being able to make and restore backup saves for each save you have in the game, Xaver allows you to make several backups per save slot. Additionally, Xaver is able to display relevant information about the save, such as:
* Name of the character
* Level the character is on
* Time the save file was last modified
* Size of the file (Save sizes can range from a couple of Megabytes in the early game to around 26 Megabytes in the late game)
Additionally, you can launch Exanima directly from the tool!

## Installation
**To install from releases (Recommended)**: Go to the releases tab and select the latest version. Then, download the file marked as Xaver_{version}_{win/linux}.zip and extract it to the directory you want to use it from. Then, launch the program using the executable and follow the instructions provided to begin using the tool.


**To install from the raw code**, you will need to have the Rust programming library installed. Once you have done this, download the code and run cargo build --release. Then, find the executable file in the 'target' folder and move it to the main folder. You can then run the code. It is possible to run the code from outside of the file's directory, but you will need to keep the images folder in the same directory as the executable.

