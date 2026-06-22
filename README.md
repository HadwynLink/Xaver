# Xaver
This tool attempts to implement saving and loading as a tool for Exanima, and was inspired by the Salvus save manager tool by steam user Silk, a link to which you can find a link to at (https://steamcommunity.com/app/362490/discussions/0/4032473829604421294/).



As of right now, Xaver can be used on the following Operating Systems:
* Linux


**WARNING: As of now, Xaver is still in the testing phase! It will not try to stop you if you accidentally delete or overwrite your save!**
If you have any issues, please report it in the issues tab and I will try to fix it by the next release.

<img width="1025" height="766" alt="Screenshot_20260617_133828" src="https://github.com/user-attachments/assets/17ba13c7-63ac-4cfa-adfb-405a207b3387" />


## Features
Along with being able to make and restore backup saves for each save you have in the game, Xaver allows you to make several backups per save slot. Additionally, Xaver is able to display relevant information about the save, such as:
* Name of the character
* Level the character is on
* Time the save file was last modified
* Size of the file (Save sizes can range from a couple of Megabytes in the early game to around 26 Megabytes in the late game)
Additionally, you can launch Exanima directly from the tool!

## Installation
To install from releases, simply download the .zip file, extract, and run Xaver. Then select your preferred save and game folders in the settings.
To install from the raw code, download the code and run cargo build --release. Then, find the executable file in target and move it to the main folder. You can then run the code.
I am aware that installation is currently not very user-friendly and am going to work on fixing that in the next release.
