# Platformer Game
A simple platformer game made in Rust with the Bevy game engine.

The point of the game is to reach the "Win" tile, while collecting as many coins along the way to increase your score as you can. There are many gaps to fall and tiles that can deal damage so be careful! The player has 30 lives and there are a total of 4 levels.

The game features 4 different levels. After winning the first level, the second level automatically loads. Winning all the levels brings the player to a win screen.

## Controls
Use the 'a' or the left arrow key to move left and 'd' or the right arrow key to move right. You can jump up to two times before touching the ground by pressing 'space' or the up arrow key.

## The Game
There are 6 types of tiles
Dirt - Solid brown tile
Grass - Solid green tile
Lava - Animated red tile that takes away one health from the player and slows them down
Bedrock - Black tile located at the bottom of the world that kills the player on contact with it
Coin - Animated yellow tile that increases the player's score when obtained (it can be obtained by touching it)
Win - A special animated tile that cycles between Green, White, and Magenta. Contact this tile to win the game.

The player's health is displayed in the top-left of the screen and the score (number of coins collected) is displayed in the top-right.

## Running
Go to the releases tab in Github and find the latest release. There is a windows binary (exe file) for windows machines. Unfortunately, if you are using a different operating system like linux, you will have to git clone this repo and use cargo to compile the code yourself.

## Pictures
1st level
![1st Level Picture](image-4.png)

Part of 2nd level
![Part of 2nd Level Picture](image-5.png)

Part of 3rd level
![Part of 3rd Level Picture](image-7.png)

Part of 4th level
![Part of 4th Level Picture](image-6.png)

Win Screen
![Win Screen](image-2.png)

Death Screen
![Death Screen](image.png)